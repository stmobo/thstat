use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, token, Attribute, Ident, LitStr, Result, Token};

use crate::numeric_enum::{ConversionError, NumericEnum};
use crate::util;
use crate::util::syn_error_from;

mod kw {
    syn::custom_keyword!(ShotType);
    syn::custom_keyword!(Stage);
    syn::custom_keyword!(Difficulty);
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
    fn into_conversion_err(self, game_id: Ident) -> ConversionError {
        match self {
            GameValueType::ShotType(_) => ConversionError::shot_type(game_id),
            GameValueType::Stage(_) => ConversionError::stage(game_id),
            GameValueType::Difficulty(_) => ConversionError::difficulty(game_id),
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
    pub fn into_numeric_enum(self, game_id: Ident) -> NumericEnum {
        NumericEnum::new(
            self.type_kw.into(),
            self.values.into_iter().map(|v| (v.ident, v.display_name)),
            self.type_kw.into_conversion_err(game_id),
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
enum GameDefItem {
    SpellId {
        _type: Token![type],
        item_kw: kw::SpellID,
        _eq: Token![=],
        ident: Ident,
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
            Ok(Self::SpellId {
                _type: input.parse()?,
                item_kw: input.parse()?,
                _eq: input.parse()?,
                ident: input.parse()?,
                _semicolon: input.parse()?,
            })
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

impl GameDefItem {}

#[derive(Debug)]
pub struct GameDefinition {
    struct_name: Ident,
    _brace: token::Brace,
    game_id: Ident,
    spell_id: Ident,
    shot_type: NumericEnum,
    stage: NumericEnum,
    difficulty: NumericEnum,
}

impl Parse for GameDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let struct_name = input.parse()?;
        let brace = braced!(content in input);

        let mut spell_id = None;
        let mut game_id = None;
        let mut shot_type = None;
        let mut stage = None;
        let mut difficulty = None;

        while !content.is_empty() {
            match content.parse()? {
                GameDefItem::SpellId { item_kw, ident, .. } => {
                    if spell_id.replace(ident).is_some() {
                        return Err(syn_error_from!(item_kw, "missing shot type definition"));
                    }
                }
                GameDefItem::GameId { item_kw, ident, .. } => {
                    if game_id.replace(ident).is_some() {
                        return Err(syn_error_from!(item_kw, "missing shot type definition"));
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
            .map(|def| def.into_numeric_enum(game_id.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing shot type definition"))?;

        let stage = stage
            .map(|def| def.into_numeric_enum(game_id.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing stage definition"))?;

        let difficulty = difficulty
            .map(|def| def.into_numeric_enum(game_id.clone()))
            .ok_or_else(|| syn_error_from!(struct_name, "missing difficulty definition"))?;

        let spell_id =
            spell_id.ok_or_else(|| syn_error_from!(struct_name, "missing spell ID type"))?;

        Ok(Self {
            struct_name,
            _brace: brace,
            game_id,
            spell_id,
            shot_type,
            stage,
            difficulty,
        })
    }
}

impl GameDefinition {
    fn define_shot_type_array(&self) -> TokenStream {
        let game_struct = &self.struct_name;
        let enum_name = self.shot_type.name();
        let n_variants = self.shot_type.variants().len();
        let elems = self.shot_type.variants().iter().map(|item| {
            let variant_name = item.name();
            quote! {
                crate::types::ShotType::new(#enum_name::#variant_name)
            }
        });

        quote! {
            impl #game_struct {
                pub const SHOT_TYPES: &[crate::types::ShotType<#game_struct>; #n_variants] = &[
                    #(#elems),*
                ];
            }
        }
    }

    fn define_wrapper_traits(
        &self,
        wrapper_type: &str,
        wrapped_type: &NumericEnum,
        array_name: &str,
    ) -> TokenStream {
        let game_struct = &self.struct_name;
        let wrapper_type: syn::Path = syn::parse_str(wrapper_type).unwrap();
        let array_ident = Ident::new(array_name, Span::call_site());
        let enum_name = wrapped_type.name();
        let n_variants = wrapped_type.variants().len();
        let elems = wrapped_type.variants().iter().map(|item| {
            let variant_name = item.name();
            quote! {
                #wrapper_type::new(#enum_name::#variant_name)
            }
        });

        quote! {
            #[automatically_derived]
            impl #wrapper_type<#game_struct> {
                pub fn iter() -> impl Iterator<Item = Self> {
                    #enum_name::iter().map(Self::new)
                }
            }

            #[automatically_derived]
            impl #game_struct {
                pub const #array_ident: &[#wrapper_type<#game_struct>; #n_variants] = &[
                    #(#elems),*
                ];
            }

            #[automatically_derived]
            impl std::borrow::Borrow<#enum_name> for #wrapper_type<#game_struct> {
                fn borrow(&self) -> &#enum_name {
                    self.as_ref()
                }
            }

            #[automatically_derived]
            impl From<#enum_name> for #wrapper_type<#game_struct> {
                fn from(value: #enum_name) -> Self {
                    Self::new(value)
                }
            }

            #[automatically_derived]
            impl From<#wrapper_type<#game_struct>> for #enum_name {
                fn from(value: #wrapper_type<#game_struct>) -> Self {
                    value.unwrap()
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

        quote! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
            pub struct #game_struct;

            impl crate::types::Game for #game_struct {
                const GAME_ID: crate::types::GameId = crate::types::GameId::#game_id;

                type SpellID = #spell_type;
                type ShotTypeID = #shot_type;
                type DifficultyID = #difficulty_type;
                type StageID = #stage_type;

                fn card_info(id: SpellId) -> &'static crate::types::SpellCardInfo<Self> {
                    id.card_info()
                }
            }
        }
    }

    pub fn to_definitions(&self) -> TokenStream {
        let mut ret = self.main_struct_def();

        ret.extend(self.shot_type.define_enum());
        ret.extend(self.stage.define_enum());
        ret.extend(self.difficulty.define_enum());
        ret.extend(self.define_wrapper_traits(
            "crate::types::ShotType",
            &self.shot_type,
            "SHOT_TYPES",
        ));
        ret.extend(self.define_wrapper_traits(
            "crate::types::Difficulty",
            &self.difficulty,
            "DIFFICULTIES",
        ));
        ret.extend(self.define_wrapper_traits("crate::types::Stage", &self.stage, "STAGES"));

        ret
    }
}
