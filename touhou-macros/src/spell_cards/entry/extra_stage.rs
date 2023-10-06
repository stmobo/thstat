use std::iter::Enumerate;

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, token, Result, Token};

use super::{NumberedName, PartialEntry, SpellGroupNumber};
use crate::spell_cards::spell_data::StageSpellType;

pub(super) struct IterExtraStageGroup {
    spell_type: StageSpellType,
    items: Enumerate<syn::punctuated::IntoIter<NumberedName>>,
}

impl Iterator for IterExtraStageGroup {
    type Item = PartialEntry<(), (), SpellGroupNumber, StageSpellType, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(|(group_num, item)| {
            PartialEntry::new(item)
                .attach_group_number(group_num as SpellGroupNumber)
                .attach_spell_type(self.spell_type)
        })
    }
}

#[derive(Clone)]
pub(super) struct ExtraStageGroup {
    spell_type: StageSpellType,
    _colon: Token![:],
    _bracket: token::Bracket,
    items: Punctuated<NumberedName, Token![,]>,
}

impl Parse for ExtraStageGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            spell_type: input.parse()?,
            _colon: input.parse()?,
            _bracket: bracketed!(content in input),
            items: content.parse_terminated(NumberedName::parse, Token![,])?,
        })
    }
}

impl IntoIterator for ExtraStageGroup {
    type Item = PartialEntry<(), (), SpellGroupNumber, StageSpellType, ()>;
    type IntoIter = IterExtraStageGroup;

    fn into_iter(self) -> Self::IntoIter {
        IterExtraStageGroup {
            spell_type: self.spell_type,
            items: self.items.into_iter().enumerate(),
        }
    }
}
