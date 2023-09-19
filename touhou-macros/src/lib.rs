#![feature(proc_macro_diagnostic)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod define_game;
mod define_memory;
mod numeric_enum;
mod spell_cards;
mod util;

use define_game::GameDefinition;
use define_memory::MemoryDef;
use numeric_enum::NumericEnum;
use spell_cards::SpellList;

#[proc_macro]
pub fn spellcards(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as SpellList)
        .into_list_tokens()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(NumericEnum, attributes(name, error_type, convert_error))]
pub fn numeric_enum(input: TokenStream) -> TokenStream {
    match NumericEnum::from_derive(parse_macro_input!(input as DeriveInput)) {
        Ok(input) => input.impl_traits(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro]
pub fn define_memory(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as MemoryDef).into_defines().into()
}

#[proc_macro]
pub fn define_game(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as GameDefinition)
        .to_definitions()
        .into()
}
