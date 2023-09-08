use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

use crate::spell_cards::spell_data::SpellEntry;

mod entry;
mod spell_data;

mod kw {
    syn::custom_keyword!(Game);
}

use entry::StageSet;

pub enum SpellListElement {
    Game {
        _kw: kw::Game,
        _colon: Token![:],
        name: Ident,
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
        } else if StageSet::peek(&lookahead) {
            input.parse().map(Self::Stage)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct SpellList {
    game: Ident,
    stages: Vec<StageSet>,
}

impl Parse for SpellList {
    fn parse(input: ParseStream) -> Result<Self> {
        let elems = input.parse_terminated(SpellListElement::parse, Token![,])?;
        let mut game = None;
        let mut seen_stages = HashSet::new();
        let mut stages = Vec::new();

        for elem in elems {
            match elem {
                SpellListElement::Game { name, .. } => {
                    if game.is_none() {
                        game = Some(name)
                    } else {
                        return Err(syn::Error::new(name.span(), "multiple games given"));
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

        if let Some(game) = game {
            Ok(Self { game, stages })
        } else {
            Err(input.error("no game given"))
        }
    }
}

impl SpellList {
    pub fn into_list_tokens(self) -> Result<TokenStream> {
        let mut entries: Vec<SpellEntry> = self.stages.into_iter().flatten().collect();

        let mut seen_locations = HashSet::new();
        for entry in &entries {
            let location = entry.location();
            if !seen_locations.insert((entry.group_number(), location)) {
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

        let mut is_duplicate_name = HashMap::new();
        for entry in &entries {
            match is_duplicate_name.entry(entry.name().to_string()) {
                Entry::Occupied(mut entry) => {
                    entry.insert(true);
                }
                Entry::Vacant(entry) => {
                    entry.insert(false);
                }
            };
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

        let game = self.game;
        let conv_iter = entries
            .into_iter()
            .map(|entry| entry.spell_def_tokens(&game, &is_duplicate_name));

        Ok(quote! { &[ #(#conv_iter),* ] })
    }
}
