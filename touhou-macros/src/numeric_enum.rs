use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitInt, LitStr, Path, Token,
    Type, Variant,
};

use crate::util;
use crate::util::syn_error_from;

#[derive(Debug, Clone)]
pub struct VariantDef(Ident, LitInt, LitStr, isize);

impl VariantDef {
    pub fn name(&self) -> &Ident {
        &self.0
    }

    pub fn discriminant(&self) -> &LitInt {
        &self.1
    }

    pub fn display_name(&self) -> &LitStr {
        &self.2
    }

    pub fn discriminant_val(&self) -> isize {
        self.3
    }
}

impl From<VariantDef> for Variant {
    fn from(value: VariantDef) -> Self {
        let eq = Token![=](value.0.span());

        Self {
            attrs: Vec::new(),
            ident: value.0,
            fields: Fields::Unit,
            discriminant: Some((
                eq,
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::Int(value.1),
                }),
            )),
        }
    }
}

impl ToTokens for VariantDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
        Token![=](self.0.span()).to_tokens(tokens);
        self.1.to_tokens(tokens);
    }
}

#[derive(Debug)]
pub enum ConversionError {
    Default {
        ident: Path,
    },
    Custom {
        err_type: Path,
        map_func: Option<Ident>,
    },
    GameValue {
        base_type: Path,
        err_type: Path,
        game_type: Ident,
        game_id: Ident,
    },
}

impl ConversionError {
    pub fn shot_type(game_id: Ident, game_type: Ident) -> Self {
        Self::GameValue {
            base_type: syn::parse_str("crate::types::errors::InvalidShotType").unwrap(),
            err_type: syn::parse_str(&format!(
                "crate::types::errors::InvalidShotType<{}>",
                game_type
            ))
            .unwrap(),
            game_type,
            game_id,
        }
    }

    pub fn difficulty(game_id: Ident, game_type: Ident) -> Self {
        Self::GameValue {
            base_type: syn::parse_str("crate::types::errors::InvalidDifficultyId").unwrap(),
            err_type: syn::parse_str(&format!(
                "crate::types::errors::InvalidDifficultyId<{}>",
                game_type
            ))
            .unwrap(),
            game_type,
            game_id,
        }
    }

    pub fn stage(game_id: Ident, game_type: Ident) -> Self {
        Self::GameValue {
            base_type: syn::parse_str("crate::types::errors::InvalidStageId").unwrap(),
            err_type: syn::parse_str(&format!(
                "crate::types::errors::InvalidStageId<{}>",
                game_type
            ))
            .unwrap(),
            game_type,
            game_id,
        }
    }

    pub fn error_ident(&self) -> &Path {
        match self {
            Self::Default { ident } => ident,
            Self::Custom { err_type, .. } | Self::GameValue { err_type, .. } => err_type,
        }
    }

    fn error_arm(&self, n_variants: usize) -> TokenStream {
        let max_range = (n_variants - 1) as u16;

        match self {
            Self::Default { ident } => quote! {
                other => Err(#ident(other as u64))
            },
            Self::Custom { map_func, .. } => quote! {
                other => Err(#map_func(other as u64))
            },
            Self::GameValue { base_type, .. } => quote! {
                other => Err(#base_type::out_of_range(other as u16, 0..=#max_range))
            },
        }
    }
}

#[derive(Debug)]
pub struct NumericEnum {
    name: Ident,
    variants: Vec<VariantDef>,
    conv_err: ConversionError,
    attrs: Vec<Attribute>,
}

impl NumericEnum {
    pub fn new<I: IntoIterator<Item = (Ident, LitStr)>>(
        name: Ident,
        variants: I,
        conv_err: ConversionError,
        attrs: Vec<Attribute>,
    ) -> Self {
        let mut variants = variants
            .into_iter()
            .enumerate()
            .map(|(idx, (var_ident, var_name))| {
                let var_discriminant = LitInt::new(&idx.to_string(), name.span());
                VariantDef(var_ident, var_discriminant, var_name, idx as isize)
            })
            .collect::<Vec<_>>();

        variants.sort_unstable_by_key(VariantDef::discriminant_val);

        Self {
            name,
            variants,
            conv_err,
            attrs,
        }
    }

    pub fn from_derive(input: DeriveInput) -> Result<Self, syn::Error> {
        if let Data::Enum(enum_data) = input.data {
            let mut variants = Vec::new();

            let conv_err = match util::parse_attribute_str("error_type", &input.attrs)? {
                Some(err_type) => ConversionError::Custom {
                    err_type,
                    map_func: util::parse_attribute_str("convert_error", &input.attrs)?,
                },
                None => ConversionError::Default {
                    ident: format_ident!("Invalid{}", &input.ident).into(),
                },
            };

            for variant in enum_data.variants {
                let variant_name = variant.ident;
                let display_name = util::attribute_as_lit_str("name", &variant.attrs)
                    .transpose()?
                    .cloned()
                    .unwrap_or_else(|| {
                        LitStr::new(
                            &util::camelcase_to_spaced(variant_name.to_string()),
                            variant_name.span(),
                        )
                    });

                if let Some((_, Expr::Lit(lit))) = variant.discriminant {
                    if let Lit::Int(value) = lit.lit {
                        let parsed_val = value.base10_parse()?;
                        variants.push(VariantDef(variant_name, value, display_name, parsed_val));
                    } else {
                        unreachable!("variant {} discriminant is not an integer", variant_name)
                    }
                } else {
                    return Err(syn_error_from!(
                        variant_name,
                        "variant does not have discriminant value"
                    ));
                }
            }

            variants.sort_unstable_by_key(VariantDef::discriminant_val);

            Ok(Self {
                name: input.ident,
                variants,
                conv_err,
                attrs: Vec::new(),
            })
        } else {
            Err(syn_error_from!(&input, "expected enum definition"))
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn err_type(&self) -> &Path {
        self.conv_err.error_ident()
    }

    pub fn variants(&self) -> &[VariantDef] {
        &self.variants[..]
    }

    fn iter_fwd_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |VariantDef(name, val, _, _)| quote!(#type_name::#name => #val))
    }

    fn iter_rev_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |VariantDef(name, val, _, _)| quote!(#val => Ok(#type_name::#name)))
    }

    fn iter_name_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |VariantDef(name, _, val, _)| quote!(#type_name::#name => #val))
    }

    fn define_error_type(&self) -> TokenStream {
        let error_name = self.conv_err.error_ident();
        let self_name = format!("\"{}\"", &self.name);

        quote! {
            #[derive(Debug, Copy, Clone)]
            pub struct #error_name(u64);

            impl std::fmt::Display for #error_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "invalid value {} for {}", self.0, #self_name)
                }
            }

            impl std::error::Error for #error_name {}
        }
    }

    fn impl_integer_conversion(&self, target_type: Type) -> TokenStream {
        let fwd_arms = self.iter_fwd_match_arms();
        let rev_arms = self.iter_rev_match_arms();
        let type_name = &self.name;
        let error_name = self.conv_err.error_ident();
        let err_arm = self.conv_err.error_arm(self.variants.len());

        quote! {
            #[automatically_derived]
            impl From<#type_name> for #target_type {
                fn from(value: #type_name) -> #target_type {
                    match value {
                        #(#fwd_arms),*
                    }
                }
            }

            #[automatically_derived]
            impl From<&#type_name> for #target_type {
                fn from(value: &#type_name) -> #target_type {
                    (*value).into()
                }
            }

            #[automatically_derived]
            impl std::convert::TryFrom<#target_type> for #type_name {
                type Error = #error_name;

                fn try_from(value: #target_type) -> Result<#type_name, #error_name> {
                    match value {
                        #(#rev_arms,)*
                        #err_arm
                    }
                }
            }

            #[automatically_derived]
            impl std::convert::TryFrom<&#target_type> for #type_name {
                type Error = #error_name;

                fn try_from(value: &#target_type) -> Result<#type_name, #error_name> {
                    #type_name::try_from(*value)
                }
            }
        }
    }

    fn impl_display(&self) -> TokenStream {
        let arms = self.iter_name_match_arms();
        let type_name = &self.name;

        quote! {
            #[automatically_derived]
            impl #type_name {
                /// Returns a fixed, human-friendly name depending on variant.
                ///
                /// This name is what is displayed when using the Display trait.
                pub fn name(&self) -> &'static str {
                    match self {
                        #(#arms),*
                    }
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.name().fmt(f)
                }
            }
        }
    }

    pub fn impl_iteration(&self) -> TokenStream {
        let self_type = &self.name;
        let iter_type = format_ident!("Iter{}", &self.name);

        let n_variants = self.variants.len();
        let _variants = self.variants.iter().map(VariantDef::name);

        let doctest_example_lines = self.variants.iter().map(VariantDef::name).map(|name| {
            quote! {
                #[doc = concat!("assert_eq!(iter.next(), Some(", stringify!(#self_type), "::", stringify!(#name), "));")]
            }
        }).take(3);

        let val_match = {
            let mut pairs = self
                .variants
                .iter()
                .map(|variant| (variant.discriminant_val(), variant.name()))
                .collect::<Vec<_>>();
            pairs.sort_unstable_by_key(|pair| pair.0);

            let arms = pairs.into_iter().map(|(idx, name)| {
                let idx = idx as usize;
                quote! { #idx => #self_type::#name }
            });

            quote! {
                #(#arms,)*
                _ => unreachable!(),
            }
        };

        quote! {
            #[doc = concat!("An iterator over all possible values for [`", stringify!(#self_type), "`].")]
            ///
            /// # Examples
            ///
            /// ```rust
            /// # use touhou::types::AllIterable;
            #[doc = concat!("# use ", module_path!(), "::", stringify!(#self_type), ";")]
            #[doc = concat!("let mut iter = ", stringify!(#self_type), "::iter_all();")]
            #(#doctest_example_lines)*
            /// ```
            #[derive(Debug, Clone)]
            pub struct #iter_type(std::ops::Range<usize>);

            #[automatically_derived]
            impl Iterator for #iter_type {
                type Item = #self_type;

                fn next(&mut self) -> Option<#self_type> {
                    self.0.next().map(|idx| match idx {
                        #val_match
                    })
                }

                #[inline]
                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.0.size_hint()
                }
            }

            #[automatically_derived]
            impl DoubleEndedIterator for #iter_type {
                fn next_back(&mut self) -> Option<#self_type> {
                    self.0.next_back().map(|idx| match idx {
                        #val_match
                    })
                }
            }

            #[automatically_derived]
            impl ExactSizeIterator for #iter_type {
                #[inline]
                fn len(&self) -> usize {
                    self.0.len()
                }
            }

            #[automatically_derived]
            impl std::iter::FusedIterator for #iter_type { }

            #[automatically_derived]
            impl crate::types::AllIterable for #self_type {
                type IterAll = #iter_type;

                fn iter_all() -> #iter_type {
                    #iter_type(0..#n_variants)
                }
            }
        }
    }

    fn impl_other_traits(&self) -> TokenStream {
        let self_type = &self.name;

        quote! {
            #[automatically_derived]
            impl Copy for #self_type { }

            #[automatically_derived]
            impl Clone for #self_type {
                #[inline]
                fn clone(&self) -> Self {
                    *self
                }
            }

            #[automatically_derived]
            impl PartialEq for #self_type {
                fn eq(&self, other: &Self) -> bool {
                    let a: u64 = self.into();
                    let b: u64 = other.into();
                    a == b
                }
            }

            #[automatically_derived]
            impl Eq for #self_type { }

            #[automatically_derived]
            impl Ord for #self_type {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    let a: u64 = self.into();
                    let b: u64 = other.into();
                    a.cmp(&b)
                }
            }

            #[automatically_derived]
            impl PartialOrd for #self_type {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            #[automatically_derived]
            impl std::hash::Hash for #self_type {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    <u64 as std::hash::Hash>::hash(&self.into(), state)
                }
            }
        }
    }

    pub fn impl_traits(&self, include_iter: bool) -> TokenStream {
        let mut ret = self.impl_display();

        if matches!(self.conv_err, ConversionError::Default { .. }) {
            ret.extend(self.define_error_type());
        }

        for type_name in [
            "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize",
        ] {
            let type_name = syn::parse_str::<Type>(type_name).unwrap();
            ret.extend(self.impl_integer_conversion(type_name))
        }

        if include_iter {
            ret.extend(self.impl_iteration());
        }

        ret.extend(self.impl_other_traits());

        if let Some(game_val_impl) = self.impl_game_value() {
            ret.extend(game_val_impl);
        }

        ret
    }

    pub fn impl_game_value(&self) -> Option<TokenStream> {
        if let ConversionError::GameValue {
            err_type,
            game_type,
            game_id,
            base_type,
            ..
        } = &self.conv_err
        {
            let name = &self.name;

            Some(quote! {
                impl crate::types::GameValue for #name {
                    type RawValue = u16;
                    type ConversionError = #err_type;

                    fn game_id(&self) -> crate::types::GameId {
                        crate::types::GameId::#game_id
                    }

                    fn raw_id(&self) -> u16 {
                        (*self).into()
                    }

                    fn from_raw(id: u16, game: crate::types::GameId) -> Result<Self, #err_type> {
                        if game == crate::types::GameId::#game_id {
                            id.try_into()
                        } else {
                            Err(#base_type::wrong_game(game))
                        }
                    }

                    fn name(&self) -> &'static str {
                        self.name()
                    }
                }
            })
        } else {
            None
        }
    }

    pub fn define_enum(&self, include_iter: bool) -> TokenStream {
        let attrs = &self.attrs;
        let name = &self.name;
        let variants = self.variants.iter().cloned().map(Variant::from);
        let trait_impl = self.impl_traits(include_iter);

        quote! {
            #(#attrs)*
            #[derive(Debug)]
            pub enum #name {
                #(#variants),*
            }

            #trait_impl
        }
    }
}
