use std::iter::{Enumerate, Flatten};

use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, token, Attribute, Result, Token};

use super::{NumberedName, PartialEntry, SpellGroupNumber, SpellIdOffset};
use crate::spell_cards::spell_data::{MainDifficulty, StageSpellType};
use crate::util::find_attribute;

#[derive(Clone)]
struct DifficultyList(Punctuated<MainDifficulty, Token![|]>);

impl Parse for DifficultyList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut difficulties = Punctuated::new();

        while !input.is_empty() {
            difficulties.push_value(input.parse()?);

            let lookahead = input.lookahead1();
            if lookahead.peek(Token![|]) {
                difficulties.push_punct(input.parse()?);
            } else {
                break;
            }
        }

        Ok(Self(difficulties))
    }
}

impl IntoIterator for DifficultyList {
    type Item = MainDifficulty;
    type IntoIter = syn::punctuated::IntoIter<MainDifficulty>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct IterSpellDifficulties(
    syn::punctuated::IntoIter<MainDifficulty>,
    NumberedName,
    SpellIdOffset,
);

impl Iterator for IterSpellDifficulties {
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, (), (), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(diff) = self.0.next() {
            let ret = PartialEntry::new(self.1.clone())
                .attach_main_difficulty(diff)
                .attach_offset(self.2);
            self.2 += 1;
            Some(ret)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct SpellWithDifficulties {
    difficulties: DifficultyList,
    _colon: Token![:],
    id_name: NumberedName,
}

impl SpellWithDifficulties {
    fn peek(lookahead: &Lookahead1) -> bool {
        MainDifficulty::peek(lookahead)
    }
}

impl Parse for SpellWithDifficulties {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            difficulties: input.parse()?,
            _colon: input.parse()?,
            id_name: input.parse()?,
        })
    }
}

impl IntoIterator for SpellWithDifficulties {
    type IntoIter = IterSpellDifficulties;
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, (), (), ()>;

    fn into_iter(self) -> Self::IntoIter {
        IterSpellDifficulties(self.difficulties.into_iter(), self.id_name, 0)
    }
}

enum IterMainStageItem {
    Single(bool, IterSpellDifficulties),
    Multi(
        bool,
        Flatten<syn::punctuated::IntoIter<SpellWithDifficulties>>,
    ),
}

impl Iterator for IterMainStageItem {
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, (), (), bool>;

    fn next(&mut self) -> Option<Self::Item> {
        let (allow_overlaps, next) = match self {
            Self::Single(allow_overlaps, inner) => (*allow_overlaps, inner.next()),
            Self::Multi(allow_overlaps, inner) => (*allow_overlaps, inner.next()),
        };

        next.map(|item| item.set_allow_overlap(allow_overlaps))
    }
}

#[derive(Clone)]
enum MainStageGroupItem {
    Single {
        allow_overlaps: bool,
        spell: SpellWithDifficulties,
    },
    Multi {
        allow_overlaps: bool,
        _brace: token::Brace,
        spells: Punctuated<SpellWithDifficulties, Token![,]>,
    },
}

impl Parse for MainStageGroupItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let allow_overlaps = find_attribute("allow_overlaps", &attrs).is_some();

        let lookahead = input.lookahead1();

        if lookahead.peek(token::Brace) {
            let content;
            Ok(Self::Multi {
                allow_overlaps,
                _brace: braced!(content in input),
                spells: content.parse_terminated(SpellWithDifficulties::parse, Token![,])?,
            })
        } else if SpellWithDifficulties::peek(&lookahead) {
            input.parse().map(move |spell| Self::Single {
                allow_overlaps,
                spell,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl IntoIterator for MainStageGroupItem {
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, (), (), bool>;
    type IntoIter = IterMainStageItem;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Single {
                allow_overlaps,
                spell,
            } => IterMainStageItem::Single(allow_overlaps, spell.into_iter()),
            Self::Multi {
                allow_overlaps,
                spells,
                ..
            } => IterMainStageItem::Multi(allow_overlaps, spells.into_iter().flatten()),
        }
    }
}

pub(super) struct IterMainStageGroup {
    spell_type: StageSpellType,
    items: Enumerate<syn::punctuated::IntoIter<MainStageGroupItem>>,
    cur_item: Option<(SpellGroupNumber, IterMainStageItem)>,
}

impl Iterator for IterMainStageGroup {
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, SpellGroupNumber, StageSpellType, bool>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((group_number, item_iter)) = &mut self.cur_item {
            if let Some(partial) = item_iter.next() {
                return Some(
                    partial
                        .attach_group_number(*group_number)
                        .attach_spell_type(self.spell_type),
                );
            }
        }

        self.cur_item = self
            .items
            .next()
            .map(|pair| (pair.0 as SpellGroupNumber, pair.1.into_iter()));

        if self.cur_item.is_some() {
            self.next()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub(super) struct MainStageGroup {
    spell_type: StageSpellType,
    _colon: Token![:],
    _bracket: token::Bracket,
    items: Punctuated<MainStageGroupItem, Token![,]>,
}

impl Parse for MainStageGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            spell_type: input.parse()?,
            _colon: input.parse()?,
            _bracket: bracketed!(content in input),
            items: content.parse_terminated(MainStageGroupItem::parse, Token![,])?,
        })
    }
}

impl IntoIterator for MainStageGroup {
    type Item = PartialEntry<MainDifficulty, SpellIdOffset, SpellGroupNumber, StageSpellType, bool>;
    type IntoIter = IterMainStageGroup;

    fn into_iter(self) -> Self::IntoIter {
        IterMainStageGroup {
            spell_type: self.spell_type,
            items: self.items.into_iter().enumerate(),
            cur_item: None,
        }
    }
}
