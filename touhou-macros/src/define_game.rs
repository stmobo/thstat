use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, token, Attribute, Ident, LitInt, LitStr, Path, Result, Token};

use crate::numeric_enum::{ConversionError, NumericEnum};
use crate::util;
use crate::util::syn_error_from;

mod kw {
    syn::custom_keyword!(ShotType);
    syn::custom_keyword!(Stage);
    syn::custom_keyword!(Difficulty);
    syn::custom_keyword!(ShotPower);
    syn::custom_keyword!(Gen1);
    syn::custom_keyword!(Gen2);
    syn::custom_keyword!(Other);
    syn::custom_keyword!(SpellID);
    syn::custom_keyword!(GAME_ID);
}

#[derive(Debug)]
struct GameValueDef {
    ident: Ident,
    display_name: LitStr,
}

impl Parse for GameValueDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();

        let display_name = if lookahead.peek(Token![:]) {
            let _: Token![:] = input.parse()?;
            input.parse()?
        } else {
            LitStr::new(&util::camelcase_to_spaced(ident.to_string()), ident.span())
        };

        Ok(Self {
            ident,
            display_name,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum GameValueType {
    ShotType(kw::ShotType),
    Stage(kw::Stage),
    Difficulty(kw::Difficulty),
}

impl GameValueType {
    fn into_conversion_err(self, game_id: Ident, game_type: Ident) -> ConversionError {
        match self {
            GameValueType::ShotType(_) => ConversionError::shot_type(game_id, game_type),
            GameValueType::Stage(_) => ConversionError::stage(game_id, game_type),
            GameValueType::Difficulty(_) => ConversionError::difficulty(game_id, game_type),
        }
    }
}

impl Parse for GameValueType {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ShotType) {
            input.parse().map(Self::ShotType)
        } else if lookahead.peek(kw::Stage) {
            input.parse().map(Self::Stage)
        } else if lookahead.peek(kw::Difficulty) {
            input.parse().map(Self::Difficulty)
        } else {
            Err(lookahead.error())
        }
    }
}

impl From<GameValueType> for Ident {
    fn from(value: GameValueType) -> Self {
        let s = match value {
            GameValueType::ShotType(_) => "ShotType",
            GameValueType::Stage(_) => "Stage",
            GameValueType::Difficulty(_) => "Difficulty",
        };

        Ident::new(s, Span::call_site())
    }
}

#[derive(Debug)]
struct GameValues {
    attrs: Vec<Attribute>,
    type_kw: GameValueType,
    _brace: token::Brace,
    values: Punctuated<GameValueDef, Token![,]>,
}

impl GameValues {
    pub fn into_numeric_enum(self, game_id: Ident, game_type: Ident) -> NumericEnum {
        NumericEnum::new(
            self.type_kw.into(),
            self.values.into_iter().map(|v| (v.ident, v.display_name)),
            self.type_kw.into_conversion_err(game_id, game_type),
            self.attrs,
        )
    }
}

impl Parse for GameValues {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let content;
        Ok(Self {
            attrs,
            type_kw: input.parse()?,
            _brace: braced!(content in input),
            values: content.parse_terminated(GameValueDef::parse, Token![,])?,
        })
    }
}

#[derive(Debug)]
enum PowerDefinition {
    Gen1(kw::Gen1),
    Gen2 {
        _item_kw: kw::Gen2,
        _paren: token::Paren,
        max: u16,
    },
    Other {
        _item_kw: kw::Other,
        _paren: token::Paren,
        type_path: Path,
    },
}

impl Parse for PowerDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Gen1) {
            input.parse().map(Self::Gen1)
        } else if lookahead.peek(kw::Gen2) {
            let content;
            Ok(Self::Gen2 {
                _item_kw: input.parse()?,
                _paren: parenthesized!(content in input),
                max: content
                    .parse::<LitInt>()
                    .and_then(|x| x.base10_parse::<u16>())?
                    * 100,
            })
        } else if lookahead.peek(kw::Other) {
            let content;
            Ok(Self::Other {
                _item_kw: input.parse()?,
                _paren: parenthesized!(content in input),
                type_path: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl PowerDefinition {
    fn power_type(&self, game_type: &Ident) -> TokenStream {
        match self {
            Self::Gen1(_) => quote! { crate::types::Gen1Power<#game_type> },
            Self::Gen2 { max, .. } => quote! { crate::types::Gen2Power<#game_type, #max> },
            Self::Other { type_path, .. } => type_path.into_token_stream(),
        }
    }
}

#[derive(Debug)]
enum GameDefItem {
    SpellId {
        _type: Token![type],
        item_kw: kw::SpellID,
        _eq: Token![=],
        ident: Ident,
        _semicolon: Token![;],
    },
    ShotPower {
        _type: Token![type],
        item_kw: kw::ShotPower,
        _eq: Token![=],
        def: PowerDefinition,
        _semicolon: Token![;],
    },
    GameId {
        _const: Token![const],
        item_kw: kw::GAME_ID,
        _eq: Token![=],
        ident: Ident,
        _semicolon: Token![;],
    },
    Values(GameValues),
}

impl Parse for GameDefItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![type]) {
            let _type = input.parse()?;
            let lookahead = input.lookahead1();

            if lookahead.peek(kw::SpellID) {
                Ok(Self::SpellId {
                    _type,
                    item_kw: input.parse()?,
                    _eq: input.parse()?,
                    ident: input.parse()?,
                    _semicolon: input.parse()?,
                })
            } else if lookahead.peek(kw::ShotPower) {
                Ok(Self::ShotPower {
                    _type,
                    item_kw: input.parse()?,
                    _eq: input.parse()?,
                    def: input.parse()?,
                    _semicolon: input.parse()?,
                })
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(Token![const]) {
            Ok(Self::GameId {
                _const: input.parse()?,
                item_kw: input.parse()?,
                _eq: input.parse()?,
                ident: input.parse()?,
                _semicolon: input.parse()?,
            })
        } else {
            input.parse().map(Self::Values)
        }
    }
}

#[derive(Debug)]
pub struct GameDefinition {
    attrs: Vec<Attribute>,
    struct_name: Ident,
    _brace: token::Brace,
    game_id: Ident,
    spell_id: Ident,
    shot_power: PowerDefinition,
    shot_type: NumericEnum,
    stage: NumericEnum,
    difficulty: NumericEnum,
}

impl Parse for GameDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let attrs = input.call(Attribute::parse_outer)?;
        let struct_name: Ident = input.parse()?;
        let brace = braced!(content in input);

        let mut spell_id = None;
        let mut game_id = None;
        let mut shot_type = None;
        let mut shot_power = None;
        let mut stage = None;
        let mut difficulty = None;

        while !content.is_empty() {
            match content.parse()? {
                GameDefItem::SpellId { item_kw, ident, .. } => {
                    if spell_id.replace(ident).is_some() {
                        return Err(syn_error_from!(item_kw, "duplicate shot type definition"));
                    }
                }
                GameDefItem::ShotPower { item_kw, def, .. } => {
                    if shot_power.replace(def).is_some() {
                        return Err(syn_error_from!(item_kw, "duplicate shot power definition"));
                    }
                }
                GameDefItem::GameId { item_kw, ident, .. } => {
                    if game_id.replace(ident).is_some() {
                        return Err(syn_error_from!(item_kw, "duplicate game ID definition"));
                    }
                }
                GameDefItem::Values(def) => match def.type_kw {
                    GameValueType::ShotType(item_kw) => {
                        if shot_type.replace(def).is_some() {
                            return Err(syn_error_from!(item_kw, "duplicate shot type definition"));
                        }
                    }
                    GameValueType::Stage(item_kw) => {
                        if stage.replace(def).is_some() {
                            return Err(syn_error_from!(item_kw, "duplicate stage definition"));
                        }
                    }
                    GameValueType::Difficulty(item_kw) => {
                        if difficulty.replace(def).is_some() {
                            return Err(syn_error_from!(
                                item_kw,
                                "duplicate difficulty definition"
                            ));
                        }
                    }
                },
            }
        }

        let game_id = game_id.ok_or_else(|| syn_error_from!(struct_name, "missing game ID"))?;

        let shot_type = shot_type
            .map(|def| def.into_numeric_enum(game_id.clone(), struct_name.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing shot type definition"))?;

        let shot_power = shot_power
            .ok_or_else(|| syn_error_from!(struct_name, "missing shot power definition"))?;

        let stage = stage
            .map(|def| def.into_numeric_enum(game_id.clone(), struct_name.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing stage definition"))?;

        let difficulty = difficulty
            .map(|def| def.into_numeric_enum(game_id.clone(), struct_name.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing difficulty definition"))?;

        let spell_id =
            spell_id.ok_or_else(|| syn_error_from!(struct_name, "missing spell ID type"))?;

        Ok(Self {
            attrs,
            struct_name,
            _brace: brace,
            game_id,
            spell_id,
            shot_type,
            stage,
            difficulty,
            shot_power,
        })
    }
}

impl GameDefinition {
    fn impl_wrapper_integer_conversion(&self, enum_type: &NumericEnum, wrapper_type: &Path, integral_type: Ident) -> TokenStream {
        let enum_name = enum_type.name();
        let conv_err = enum_type.err_type();
        let game_struct = &self.struct_name;

        quote! {
            #[automatically_derived]
            impl From<#wrapper_type<#game_struct>> for #integral_type {
                fn from(value: #wrapper_type<#game_struct>) -> #integral_type {
                    value.unwrap().into()
                }
            }

            #[automatically_derived]
            impl From<&#wrapper_type<#game_struct>> for #integral_type {
                fn from(value: &#wrapper_type<#game_struct>) -> #integral_type {
                    value.unwrap().into()
                }
            }

            #[automatically_derived]
            impl TryFrom<#integral_type> for #wrapper_type<#game_struct> {
                type Error = #conv_err;

                fn try_from(value: #integral_type) -> Result<#wrapper_type<#game_struct>, #conv_err> {
                    #enum_name::try_from(value).map(#wrapper_type::new)
                }
            }

            #[automatically_derived]
            impl TryFrom<&#integral_type> for #wrapper_type<#game_struct> {
                type Error = #conv_err;

                fn try_from(value: &#integral_type) -> Result<#wrapper_type<#game_struct>, #conv_err> {
                    #enum_name::try_from(value).map(#wrapper_type::new)
                }
            }
        }
    }

    fn define_numeric_enums(&self, items: &[(&NumericEnum, &str, &str)]) -> TokenStream {


        let game_struct = &self.struct_name;
        let mut main_defs = TokenStream::new();
        let mut iter_defs = TokenStream::new();

        for &(enum_type, wrapper_base_name, array_name) in items {
            let enum_name = enum_type.name();
            let n_variants = enum_type.variants().len();
            let array_ident = Ident::new(array_name, enum_name.span());
            let main_wrapper_type: syn::Path =
                syn::parse_str(&format!("crate::types::{}", wrapper_base_name)).unwrap();
            let any_wrapper_type: syn::Path =
                syn::parse_str(&format!("crate::types::any::Any{}", wrapper_base_name)).unwrap();
            let conv_error_type = enum_type.err_type();

            let array_elems = enum_type.variants().iter().map(|item| {
                let variant_name = item.name();
                quote! {
                    #main_wrapper_type::new(#enum_name::#variant_name)
                }
            });

            let int_conversions = [
                "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize",
            ].into_iter().map(
                |s| self.impl_wrapper_integer_conversion(enum_type, &main_wrapper_type, Ident::new(s, Span::call_site()))
            );

            main_defs.extend(enum_type.define_enum(false));
            iter_defs.extend(enum_type.impl_iteration());

            main_defs.extend(quote! {
                #[automatically_derived]
                impl #game_struct {
                    pub const #array_ident: [#main_wrapper_type<#game_struct>; #n_variants] = [
                        #(#array_elems),*
                    ];
                }

                #[automatically_derived]
                impl PartialEq<#main_wrapper_type<#game_struct>> for #enum_name {
                    fn eq(&self, other: &#main_wrapper_type<#game_struct>) -> bool {
                        self.eq(&other.unwrap())
                    }
                }

                #[automatically_derived]
                impl PartialEq<#enum_name> for #main_wrapper_type<#game_struct> {
                    fn eq(&self, other: &#enum_name) -> bool {
                        self.unwrap().eq(other)
                    }
                }
        
                #[automatically_derived]
                impl PartialOrd<#main_wrapper_type<#game_struct>> for #enum_name {
                    fn partial_cmp(&self, other: &#main_wrapper_type<#game_struct>) -> Option<std::cmp::Ordering> {
                        Some(self.cmp(&other.unwrap()))
                    }
                }

                #[automatically_derived]
                impl PartialOrd<#enum_name> for #main_wrapper_type<#game_struct> {
                    fn partial_cmp(&self, other: &#enum_name) -> Option<std::cmp::Ordering> {
                        Some(self.unwrap().cmp(other))
                    }
                }

                #[automatically_derived]
                impl std::borrow::Borrow<#enum_name> for #main_wrapper_type<#game_struct> {
                    fn borrow(&self) -> &#enum_name {
                        self.as_ref()
                    }
                }

                #[automatically_derived]
                impl From<#enum_name> for #main_wrapper_type<#game_struct> {
                    fn from(value: #enum_name) -> Self {
                        Self::new(value)
                    }
                }

                #[automatically_derived]
                impl From<#main_wrapper_type<#game_struct>> for #enum_name {
                    fn from(value: #main_wrapper_type<#game_struct>) -> Self {
                        value.unwrap()
                    }
                }

                #[automatically_derived]
                impl TryFrom<#any_wrapper_type> for #enum_name {
                    type Error = #conv_error_type;

                    fn try_from(value: #any_wrapper_type) -> Result<Self, Self::Error> {
                        value.downcast_id::<#game_struct>()
                    }
                }

                #[automatically_derived]
                impl From<#enum_name> for #any_wrapper_type {
                    fn from(value: #enum_name) -> Self {
                        Self::new::<#game_struct>(value)
                    }
                }

                #(#int_conversions)*
            })
        }

        quote! {
            #main_defs

            pub mod iter {
                use super::*;

                #iter_defs
            }
        }
    }

    fn define_shot_power_traits(&self) -> TokenStream {
        let game_struct = &self.struct_name;
        let shot_power_type = self.shot_power.power_type(game_struct);

        quote! {
            #[automatically_derived]
            impl std::borrow::Borrow<#shot_power_type> for crate::types::ShotPower<#game_struct> {
                fn borrow(&self) -> &#shot_power_type {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl From<#shot_power_type> for crate::types::ShotPower<#game_struct> {
                fn from(value: #shot_power_type) -> Self {
                    Self::new(value)
                }
            }

            #[automatically_derived]
            impl From<crate::types::ShotPower<#game_struct>> for #shot_power_type {
                fn from(value: crate::types::ShotPower<#game_struct>) -> Self {
                    value.unwrap()
                }
            }

            #[automatically_derived]
            impl PartialEq<crate::types::ShotPower<#game_struct>> for #shot_power_type {
                fn eq(&self, other: &crate::types::ShotPower<#game_struct>) -> bool {
                    self.eq(&other.unwrap())
                }
            }

            #[automatically_derived]
            impl PartialEq<#shot_power_type> for crate::types::ShotPower<#game_struct> {
                fn eq(&self, other: &#shot_power_type) -> bool {
                    self.unwrap().eq(other)
                }
            }
    
            #[automatically_derived]
            impl PartialOrd<crate::types::ShotPower<#game_struct>> for #shot_power_type {
                fn partial_cmp(&self, other: &crate::types::ShotPower<#game_struct>) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(&other.unwrap()))
                }
            }

            #[automatically_derived]
            impl PartialOrd<#shot_power_type> for crate::types::ShotPower<#game_struct> {
                fn partial_cmp(&self, other: &#shot_power_type) -> Option<std::cmp::Ordering> {
                    Some(self.unwrap().cmp(other))
                }
            }
        }
    }

    fn main_struct_def(&self) -> TokenStream {
        let game_struct = &self.struct_name;
        let game_id = &self.game_id;
        let spell_type = &self.spell_id;
        let shot_type = self.shot_type.name();
        let stage_type = self.stage.name();
        let difficulty_type = self.difficulty.name();
        let power_type = self.shot_power.power_type(game_struct);
        let attrs = &self.attrs;

        quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, serde::Serialize, serde::Deserialize)]
            #(#attrs)*
            pub struct #game_struct;

            impl crate::types::Game for #game_struct {
                const GAME_ID: crate::types::GameId = GameId::#game_id;

                type SpellID = #spell_type;
                type ShotTypeID = #shot_type;
                type DifficultyID = #difficulty_type;
                type StageID = #stage_type;
                type ShotPower = #power_type;

                fn card_info(id: SpellId) -> &'static crate::types::SpellCardInfo<Self> {
                    id.card_info()
                }
            }
        }
    }

    pub fn to_definitions(&self) -> TokenStream {
        let mut ret = self.main_struct_def();
        ret.extend(self.define_numeric_enums(&[
            (&self.shot_type, "ShotType", "SHOT_TYPES"),
            (&self.difficulty, "Difficulty", "DIFFICULTIES"),
            (&self.stage, "Stage", "STAGES"),
        ]));

        ret.extend(self.define_shot_power_traits());

        ret
    }
}
