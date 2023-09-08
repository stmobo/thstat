use std::collections::hash_map::Entry;
use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Ident, LitInt, LitStr, Result, Token};

use crate::spell_cards::spell_data::SpellEntry;

mod flat_entry;
mod grouped_entry;
mod spell_data;

mod kw {
    syn::custom_keyword!(Game);
}

pub enum SpellListElement {
    Flat(flat_entry::FlatEntry),
    Group(grouped_entry::StageSet),
}

impl Parse for SpellListElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![#]) || lookahead.peek(LitInt) {
            input.parse().map(Self::Flat)
        } else {
            input.parse().map(Self::Group)
        }
    }
}

pub struct SpellList {
    game_token: kw::Game,
    colon: Token![:],
    game_ident: Ident,
    comma: Token![,],
    entries: Punctuated<SpellListElement, Token![,]>,
}

impl SpellList {
    pub fn into_list_tokens(self) -> TokenStream {
        let mut entries: Vec<SpellEntry> = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            match entry {
                SpellListElement::Flat(entry) => entries.push(entry.into_spell_entry()),
                SpellListElement::Group(group) => entries.extend(group.into_spell_entries()),
            };
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
        let mut out_tokens = Vec::with_capacity(entries.len());

        for (i, entry) in entries.into_iter().enumerate() {
            let expected_id = (i as u32) + first_id;
            if entry.id() != expected_id {
                let msg = format!("duplicate or missing spell ID {}", expected_id);
                return quote! { compile_error!(#msg) };
            }

            out_tokens.push(entry.spell_def_tokens(&self.game_ident, &is_duplicate_name));
        }

        quote! { &[ #(#out_tokens),* ] }
    }
}

impl Parse for SpellList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            game_token: input.parse()?,
            colon: input.parse()?,
            game_ident: input.parse()?,
            comma: input.parse()?,
            entries: input.parse_terminated(SpellListElement::parse, Token![,])?,
        })
    }
}
