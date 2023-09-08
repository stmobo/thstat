use std::collections::hash_map::Entry;
use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Ident, LitInt, LitStr, Result, Token};

use super::spell_data::{SpellEntry, SpellLocation};

#[allow(dead_code)]
#[derive(Clone)]
pub struct FlatEntry {
    hash: Option<Token![#]>,
    spell_number: LitInt,
    paren_token: token::Paren,
    location: SpellLocation,
    colon_token: Token![:],
    spell_name: LitStr,
}

impl FlatEntry {
    pub fn into_spell_entry(self) -> SpellEntry {
        SpellEntry::new(self.location, self.spell_number, self.spell_name, 0)
    }
}

impl Parse for FlatEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            hash: input.parse()?,
            spell_number: input.parse()?,
            paren_token: parenthesized!(content in input),
            location: content.parse()?,
            colon_token: input.parse()?,
            spell_name: input.parse()?,
        })
    }
}
