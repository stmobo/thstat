#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod numeric_enum;
mod spell_cards;

use numeric_enum::NumericEnum;
use spell_cards::SpellList;

#[proc_macro]
pub fn spellcards(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as SpellList)
        .into_list_tokens()
        .into()
}

#[proc_macro_derive(NumericEnum, attributes(name, error_type, convert_error))]
pub fn numeric_enum(input: TokenStream) -> TokenStream {
    match NumericEnum::new(parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input.impl_traits(),
        Err(err) => err.to_compile_error(),
    }
    .into()
}
