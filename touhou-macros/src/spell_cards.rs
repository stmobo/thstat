use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Result, Token, Attribute};

use crate::util::syn_error_from;

mod entry;
mod spell_data;

mod kw {
    syn::custom_keyword!(Game);
    syn::custom_keyword!(Expected);
}

use entry::{StageSet, SpellEntry};

pub enum SpellListElement {
    Game {
        _kw: kw::Game,
        _colon: Token![:],
        name: Ident,
    },
    Expected {
        _kw: kw::Expected,
        _colon: Token![:],
        count: LitInt,
    },
    Stage(StageSet),
}

impl Parse for SpellListElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::Game) {
            Ok(Self::Game {
                _kw: input.parse()?,
                _colon: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::Expected) {
            Ok(Self::Expected {
                _kw: input.parse()?,
                _colon: input.parse()?,
                count: input.parse()?,
            })
        } else if StageSet::peek(&lookahead) {
            input.parse().map(Self::Stage)
        } else {
            Err(lookahead.error())
        }
    }
}

struct SpellsDef {
    attrs: Vec<Attribute>,
    game: Ident,
    expected: LitInt,
    stages: Vec<StageSet>,
}

impl Parse for SpellsDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let elems = input.parse_terminated(SpellListElement::parse, Token![,])?;
        let mut game = None;
        let mut seen_stages = HashSet::new();
        let mut stages = Vec::new();
        let mut expected = None;

        for elem in elems {
            match elem {
                SpellListElement::Game { name, .. } => {
                    if game.is_none() {
                        game = Some(name)
                    } else {
                        return Err(syn::Error::new(name.span(), "multiple games given"));
                    }
                }
                SpellListElement::Expected { count, .. } => {
                    if expected.is_none() {
                        expected = Some(count);
                    } else {
                        return Err(syn::Error::new(
                            count.span(),
                            "multiple expected counts given",
                        ));
                    }
                }
                SpellListElement::Stage(stage_set) => {
                    let stage = stage_set.stage();
                    if !seen_stages.insert(stage) {
                        return Err(syn::Error::new(
                            stage.span(),
                            format!("Multiple spell sets given for stage {}", stage),
                        ));
                    }

                    stages.push(stage_set);
                }
            }
        }

        Ok(Self {
            attrs,
            game: game.ok_or_else(|| input.error("missing 'Game'"))?,
            expected: expected.ok_or_else(|| input.error("missing 'Expected'"))?,
            stages,
        })
    }
}

pub struct SpellList {
    attrs: Vec<Attribute>,
    game: Ident,
    entries: Vec<SpellEntry>
}

impl SpellList {
    fn new(def: SpellsDef) -> Result<Self> {
        let mut entries: Vec<SpellEntry> = def.stages.into_iter().flatten().collect();
        let n_entries = entries.len();
        let expected: usize = def.expected.base10_parse()?;

        if entries.len() != expected {
            return Err(syn_error_from!(
                def.expected,
                "Incorrect number of spells defined (found {}, expected {})",
                n_entries,
                expected
            ));
        }

        let mut seen_locations = HashSet::new();
        for entry in &entries {
            let location = entry.location();
            if !seen_locations.insert((entry.group_number(), location)) && !entry.allow_overlap() {
                location
                    .difficulty_span()
                    .unwrap()
                    .warning("Spell cards overlap in sequence number and difficulty")
                    .emit();
            }
        }

        for pair in entries.windows(2) {
            if pair[1].id() != (pair[0].id() + 1) {
                pair[1]
                    .id_span()
                    .span()
                    .unwrap()
                    .warning("Spell numbers are not consecutive")
                    .emit();
            }
        }

        entries.sort_unstable_by_key(|entry| entry.id());

        let first_id = entries.first().unwrap().id();
        for (i, entry) in entries.iter().enumerate() {
            let expected_id = (i as u32) + first_id;
            if entry.id() != expected_id {
                return Err(syn::Error::new(
                    entry.id_span().span(),
                    format!("duplicate or missing spell ID {}", expected_id),
                ));
            }
        }

        Ok(Self {
            attrs: def.attrs,
            game: def.game,
            entries
        })
    }

    pub fn define_spell_data(&self) -> TokenStream {
        static CONVERT_TYPES: &[&str; 8] = &["u16", "u32", "u64", "usize", "i16", "i32", "i64", "isize"];

        let game = &self.game;
        let attrs = &self.attrs;
        let n_cards = self.entries.len() as u16;
        let n_cards_u32 = self.entries.len() as u32;
        let n_cards_usize = self.entries.len();
        let spells = self.entries.iter().map(move |entry| entry.spell_def_tokens(game));

        let conversions = CONVERT_TYPES.iter().map(|name| {
            let type_ident = Ident::new(name, Span::call_site());
            quote! {
                #[automatically_derived]
                impl From<SpellId> for #type_ident {
                    fn from(value: SpellId) -> Self {
                        value.0.get() as #type_ident
                    }
                }
                
                #[automatically_derived]
                impl TryFrom<#type_ident> for SpellId {
                    type Error = crate::types::InvalidCardId;
                
                    fn try_from(value: #type_ident) -> Result<Self, Self::Error> {
                        <u16 as TryFrom<#type_ident>>::try_from(value)
                            .map_err(|_| crate::types::InvalidCardId::InvalidCard(<#game as crate::types::Game>::GAME_ID, value as u32, #n_cards_u32))
                            .and_then(Self::new)
                    }
                }
            }
        });

        quote! {
            const SPELL_CARDS: &[crate::types::SpellCardInfo<#game>; #n_cards_usize] = &[ #(#spells),* ];

            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
            #[repr(transparent)]
            #(#attrs)*
            ///
            /// This type automatically dereferences to a [`crate::types::SpellCardInfo`] containing information about the given spell, such as its name and the stage in which it appears. 
            pub struct SpellId(std::num::NonZeroU16);

            #[automatically_derived]
            impl SpellId {
                /// Creates a new `SpellId` if the value represents a valid spell.
                /// 
                #[doc = concat!("Valid spell IDs range from 1 to ", stringify!(#n_cards), ", inclusive.")]
                pub const fn new(value: u16) -> Result<Self, crate::types::InvalidCardId> {
                    if value <= #n_cards {
                        if let Some(value) = std::num::NonZeroU16::new(value) {
                            return Ok(Self(value));
                        }
                    }

                    Err(crate::types::InvalidCardId::InvalidCard(
                        <#game as crate::types::Game>::GAME_ID,
                        value as u32,
                        #n_cards_u32,
                    ))
                }

                /// Gets a reference to static information for this spell.
                pub const fn card_info(&self) -> &'static crate::types::SpellCardInfo<#game> {
                    &SPELL_CARDS[(self.0.get() - 1) as usize]
                }

                /// Gets the inner numeric ID value.
                pub const fn unwrap(self) -> u16 {
                    self.0.get()
                }

                /// Returns an iterator over every spell card in the game.
                pub fn iter_all() -> impl Iterator<Item = Self> {
                    (1..=#n_cards).map(|i| Self::new(i).unwrap())
                }
            }

            #(#conversions)*

            #[automatically_derived]
            impl std::convert::AsRef<crate::types::SpellCardInfo<#game>> for SpellId {
                fn as_ref(&self) -> &crate::types::SpellCardInfo<#game> {
                    &SPELL_CARDS[(self.0.get() - 1) as usize]
                }
            }

            #[automatically_derived]
            impl std::ops::Deref for SpellId {
                type Target = crate::types::SpellCardInfo<#game>;

                fn deref(&self) -> &Self::Target {
                    &SPELL_CARDS[(self.0.get() - 1) as usize]
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for SpellId {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.0.fmt(f)
                }
            }

            #[automatically_derived]
            impl crate::types::GameValue for SpellId {
                type RawValue = u32;
                type ConversionError = crate::types::InvalidCardId;
            
                fn game_id(&self) -> crate::types::GameId {
                    <#game as crate::types::Game>::GAME_ID
                }
            
                fn raw_id(&self) -> u32 {
                    (*self).into()
                }
            
                fn from_raw(id: u32, game: crate::types::GameId) -> Result<Self, Self::ConversionError> {
                    if game == <#game as crate::types::Game>::GAME_ID {
                        id.try_into()
                    } else {
                        Err(crate::types::InvalidCardId::UnexpectedGameId(game, <#game as crate::types::Game>::GAME_ID))
                    }
                }
            
                fn name(&self) -> &'static str {
                    self.card_info().name
                }
            }
            
            #[automatically_derived]
            impl From<SpellId> for crate::types::SpellCard<#game> {
                fn from(value: SpellId) -> Self {
                    Self::new(value)
                }
            }
            
            #[automatically_derived]
            impl std::borrow::Borrow<SpellId> for crate::types::SpellCard<#game> {
                fn borrow(&self) -> &SpellId {
                    self.as_ref()
                }
            }
            
            #[automatically_derived]
            impl TryFrom<crate::types::any::AnySpellCard> for SpellId {
                type Error = crate::types::errors::InvalidCardId;

                fn try_from(value: crate::types::any::AnySpellCard) -> Result<Self, Self::Error> {
                    value.downcast_id::<#game>()
                }
            }

            #[automatically_derived]
            impl From<SpellId> for crate::types::any::AnySpellCard {
                fn from(value: SpellId) -> Self {
                    Self::new::<#game>(value)
                }
            }
        }
    }
}

impl Parse for SpellList {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().and_then(Self::new)
    }
}
