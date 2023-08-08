#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod spell_cards;

use spell_cards::SpellList;

#[proc_macro]
pub fn spellcards(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as SpellList)
        .to_list_tokens()
        .into()
}
