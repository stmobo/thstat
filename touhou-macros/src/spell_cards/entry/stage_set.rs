use std::iter::{Enumerate, Flatten};

use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, token, Result, Token};

use super::{ExtraStageGroup, MainStageGroup, NumberedName, SpellEntry, SpellGroupNumber};
use crate::spell_cards::spell_data::{kw, ExtraStage, MainStage, SpellLocation, Stage};

#[allow(clippy::large_enum_variant)]
enum IterStageSetInner {
    Main {
        stage: MainStage,
        inner: Flatten<syn::punctuated::IntoIter<MainStageGroup>>,
    },
    Extra {
        stage: ExtraStage,
        inner: Flatten<syn::punctuated::IntoIter<ExtraStageGroup>>,
    },
    LastWord {
        lw: kw::LastWord,
        inner: Enumerate<syn::punctuated::IntoIter<NumberedName>>,
    },
}

impl Iterator for IterStageSetInner {
    type Item = SpellEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Main { stage, inner } => inner
                .next()
                .map(|partial| partial.into_main_stage_entry(*stage)),
            Self::Extra { stage, inner } => inner
                .next()
                .map(|partial| partial.into_extra_stage_entry(*stage)),
            Self::LastWord { lw, inner } => inner.next().map(|(group_num, id_name)| {
                SpellEntry::new(
                    SpellLocation::LastWord(*lw),
                    id_name.spell_id,
                    id_name.name,
                    0,
                    group_num as SpellGroupNumber,
                    false,
                )
            }),
        }
    }
}

#[derive(Clone)]
enum StageSetInner {
    Main {
        stage: MainStage,
        _colon: Token![:],
        _brace: token::Brace,
        groups: Punctuated<MainStageGroup, Token![,]>,
    },
    Extra {
        stage: ExtraStage,
        _colon: Token![:],
        _brace: token::Brace,
        groups: Punctuated<ExtraStageGroup, Token![,]>,
    },
    LastWord {
        lw: kw::LastWord,
        _colon: Token![:],
        _bracket: token::Bracket,
        spells: Punctuated<NumberedName, Token![,]>,
    },
}

impl Parse for StageSetInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let content;
        if MainStage::peek(&lookahead) {
            Ok(Self::Main {
                stage: input.parse()?,
                _colon: input.parse()?,
                _brace: braced!(content in input),
                groups: content.parse_terminated(MainStageGroup::parse, Token![,])?,
            })
        } else if ExtraStage::peek(&lookahead) {
            Ok(Self::Extra {
                stage: input.parse()?,
                _colon: input.parse()?,
                _brace: braced!(content in input),
                groups: content.parse_terminated(ExtraStageGroup::parse, Token![,])?,
            })
        } else if lookahead.peek(kw::LastWord) {
            Ok(Self::LastWord {
                lw: input.parse()?,
                _colon: input.parse()?,
                _bracket: bracketed!(content in input),
                spells: content.parse_terminated(NumberedName::parse, Token![,])?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl IntoIterator for StageSetInner {
    type Item = SpellEntry;
    type IntoIter = IterStageSetInner;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Main { stage, groups, .. } => IterStageSetInner::Main {
                stage,
                inner: groups.into_iter().flatten(),
            },
            Self::Extra { stage, groups, .. } => IterStageSetInner::Extra {
                stage,
                inner: groups.into_iter().flatten(),
            },
            Self::LastWord { lw, spells, .. } => IterStageSetInner::LastWord {
                lw,
                inner: spells.into_iter().enumerate(),
            },
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct StageSet(StageSetInner);

impl StageSet {
    pub fn peek(lookahead: &Lookahead1) -> bool {
        MainStage::peek(lookahead) || ExtraStage::peek(lookahead) || lookahead.peek(kw::LastWord)
    }

    pub fn stage(&self) -> Stage {
        match &self.0 {
            StageSetInner::Main { stage, .. } => (*stage).into(),
            StageSetInner::Extra { stage, .. } => (*stage).into(),
            StageSetInner::LastWord { lw, .. } => (*lw).into(),
        }
    }
}

impl Parse for StageSet {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Self)
    }
}

#[repr(transparent)]
pub struct IterStageSet(IterStageSetInner);

impl Iterator for IterStageSet {
    type Item = SpellEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl IntoIterator for StageSet {
    type Item = SpellEntry;
    type IntoIter = IterStageSet;

    fn into_iter(self) -> Self::IntoIter {
        IterStageSet(self.0.into_iter())
    }
}
