use std::error::Error;
use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Expr, Ident, Lit, LitInt, LitStr, Meta, Type};

#[derive(Debug, Clone)]
pub struct InvalidNumericEnum(String);

impl Display for InvalidNumericEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for InvalidNumericEnum {}

impl InvalidNumericEnum {
    pub fn into_compile_error(self) -> TokenStream {
        let msg = self.0;
        quote! { compile_error!(#msg) }
    }
}

impl From<String> for InvalidNumericEnum {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&'static str> for InvalidNumericEnum {
    fn from(value: &'static str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug)]
pub struct NumericEnum {
    name: Ident,
    variants: Vec<(Ident, LitInt, LitStr)>,
    map_conv_err: Option<Ident>,
    err_type: Option<Ident>,
}

impl NumericEnum {
    fn get_ident_attr<'a, T, U>(
        attr_name: &'static str,
        attrs: T,
    ) -> Result<Option<U>, InvalidNumericEnum>
    where
        T: IntoIterator<Item = &'a Attribute>,
        U: syn::parse::Parse,
    {
        attrs
            .into_iter()
            .find_map(|attr| {
                if let Meta::NameValue(kv) = &attr.meta {
                    if kv.path.is_ident(attr_name) {
                        if let Expr::Lit(lit) = &kv.value {
                            if let Lit::Str(val) = &lit.lit {
                                let val = val.value();
                                return Some(
                                    syn::parse_str(&val)
                                        .map_err(|x| InvalidNumericEnum(x.to_string())),
                                );
                            }
                        } else {
                            return Some(Err(format!(
                                "expected literal string for attribute {}",
                                attr_name
                            )
                            .into()));
                        }
                    }
                }

                None
            })
            .transpose()
    }

    pub fn new(input: DeriveInput) -> Result<Self, InvalidNumericEnum> {
        if let Data::Enum(enum_data) = input.data {
            let mut variants = Vec::new();

            let err_type = Self::get_ident_attr("error_type", &input.attrs)?;
            let map_conv_err = Self::get_ident_attr("convert_error", &input.attrs)?;

            for variant in enum_data.variants {
                let variant_name = variant.ident;
                let display_name = variant
                    .attrs
                    .into_iter()
                    .find_map(|attr| {
                        if let Meta::NameValue(kv) = attr.meta {
                            if kv.path.is_ident("name") {
                                if let Expr::Lit(lit) = kv.value {
                                    if let Lit::Str(display_name) = lit.lit {
                                        return Some(Ok(display_name));
                                    }
                                } else {
                                    return Some(Err(format!(
                                        "expected literal string for display name for variant {}",
                                        variant_name
                                    )));
                                }
                            }
                        }

                        None
                    })
                    .unwrap_or_else(|| {
                        let s = format!("\"{}\"", &variant_name);
                        Ok(syn::parse_str::<LitStr>(&s).unwrap())
                    })
                    .map_err(InvalidNumericEnum::from)?;

                if let Some((_, Expr::Lit(lit))) = variant.discriminant {
                    if let Lit::Int(value) = lit.lit {
                        variants.push((variant_name, value, display_name));
                    } else {
                        return Err(format!(
                            "variant {} does not have integer value",
                            variant_name
                        )
                        .into());
                    }
                } else {
                    return Err(format!(
                        "variant {} does not have discriminant value",
                        variant_name
                    )
                    .into());
                }
            }

            Ok(Self {
                name: input.ident,
                variants,
                map_conv_err,
                err_type,
            })
        } else {
            Err("expected enum definition".into())
        }
    }

    fn iter_fwd_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |(name, val, _)| quote!(#type_name::#name => #val))
    }

    fn iter_rev_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |(name, val, _)| quote!(#val => Ok(#type_name::#name)))
    }

    fn iter_name_match_arms(&self) -> impl Iterator<Item = TokenStream> + '_ {
        let type_name = &self.name;
        self.variants
            .iter()
            .map(move |(name, _, val)| quote!(#type_name::#name => #val))
    }

    fn error_ident(&self) -> Ident {
        self.err_type
            .clone()
            .unwrap_or_else(|| format_ident!("Invalid{}", &self.name))
    }

    fn define_error_type(&self) -> TokenStream {
        let error_name = self.error_ident();
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
        let error_name = self.error_ident();

        let err_arm = self
            .map_conv_err
            .as_ref()
            .map(|map_fn| {
                quote! {
                    other => Err(#map_fn(other as u64))
                }
            })
            .unwrap_or_else(|| {
                quote! {
                    other => Err(#error_name(other as u64))
                }
            });

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
                pub fn name(&self) -> &'static str {
                    match self {
                        #(#arms),*
                    }
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str(self.name())
                }
            }
        }
    }

    fn impl_iteration(&self) -> TokenStream {
        let self_type = &self.name;
        let iter_type = format_ident!("Iter{}", &self.name);

        let mut arms = TokenStream::new();
        let mut size_hint_arms = TokenStream::new();
        for (i, x) in self.variants.windows(2).enumerate() {
            let prev_variant = &x[0].0;
            let next_variant = &x[1].0;
            let n_remaining = self.variants.len() - i;

            arms.extend(
                quote!(Some(#self_type::#prev_variant) => Some(#self_type::#next_variant),),
            );
            size_hint_arms.extend(
                quote!(Some(#self_type::#prev_variant) => (#n_remaining, Some(#n_remaining)),),
            );
        }

        let last_variant = &self.variants.last().unwrap().0;
        arms.extend(quote!(Some(#self_type::#last_variant) => None,));
        size_hint_arms.extend(quote!(Some(#self_type::#last_variant) => (0, Some(0)),));

        let first_variant = &self.variants.first().unwrap().0;

        quote! {
            #[derive(Debug, Copy, Clone)]
            pub struct #iter_type(Option<#self_type>);

            impl Iterator for #iter_type {
                type Item = #self_type;

                fn next(&mut self) -> Option<#self_type> {
                    let ret = self.0.take();
                    self.0 = match ret {
                        #arms
                        None => None
                    };
                    ret
                }
            }

            #[automatically_derived]
            impl #self_type {
                pub fn iter() -> #iter_type {
                    #iter_type(Some(#self_type::#first_variant))
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

    pub fn impl_traits(&self) -> TokenStream {
        let mut ret = self.impl_display();

        if self.err_type.is_none() {
            ret.extend(self.define_error_type());
        }

        for type_name in [
            "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize",
        ] {
            let type_name = syn::parse_str::<Type>(type_name).unwrap();
            ret.extend(self.impl_integer_conversion(type_name))
        }

        ret.extend(self.impl_iteration());
        ret.extend(self.impl_other_traits());

        ret
    }
}
