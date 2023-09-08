use std::iter::{Enumerate, Flatten};

use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, token, LitInt, LitStr, Result, Token};

use super::spell_data::{
    kw, ExtraStage, MainDifficulty, MainStage, SpellEntry, SpellLocation, Stage, StageSpellType,
};

type SpellIdOffset = u32;
type SpellGroupNumber = u32;

#[derive(Clone)]
struct NumberedName {
    _hash: Option<Token![#]>,
    spell_id: LitInt,
    name: LitStr,
}

impl Parse for NumberedName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            _hash: input.parse()?,
            spell_id: input.parse()?,
            name: input.parse()?,
        })
    }
}

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

struct IterSpellDifficulties(syn::punctuated::IntoIter<MainDifficulty>, NumberedName, u32);

impl Iterator for IterSpellDifficulties {
    type Item = (MainDifficulty, NumberedName, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(diff) = self.0.next() {
            let ret = Some((diff, self.1.clone(), self.2));
            self.2 += 1;
            ret
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
    type Item = (MainDifficulty, NumberedName, SpellIdOffset);

    fn into_iter(self) -> Self::IntoIter {
        IterSpellDifficulties(self.difficulties.into_iter(), self.id_name, 0)
    }
}

enum IterMainStageItem {
    Single(IterSpellDifficulties),
    Multi(Flatten<syn::punctuated::IntoIter<SpellWithDifficulties>>),
}

impl Iterator for IterMainStageItem {
    type Item = (MainDifficulty, NumberedName, SpellIdOffset);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(inner) => inner.next(),
            Self::Multi(inner) => inner.next(),
        }
    }
}

#[derive(Clone)]
enum MainStageGroupItem {
    Single(SpellWithDifficulties),
    Multi {
        _brace: token::Brace,
        spells: Punctuated<SpellWithDifficulties, Token![,]>,
    },
}

impl Parse for MainStageGroupItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(token::Brace) {
            let content;
            Ok(Self::Multi {
                _brace: braced!(content in input),
                spells: content.parse_terminated(SpellWithDifficulties::parse, Token![,])?,
            })
        } else if SpellWithDifficulties::peek(&lookahead) {
            input.parse().map(Self::Single)
        } else {
            Err(lookahead.error())
        }
    }
}

impl IntoIterator for MainStageGroupItem {
    type Item = (MainDifficulty, NumberedName, SpellIdOffset);
    type IntoIter = IterMainStageItem;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Single(inner) => IterMainStageItem::Single(inner.into_iter()),
            Self::Multi { spells, .. } => IterMainStageItem::Multi(spells.into_iter().flatten()),
        }
    }
}

struct IterMainStageGroup {
    spell_type: StageSpellType,
    items: Enumerate<syn::punctuated::IntoIter<MainStageGroupItem>>,
    cur_item: Option<(SpellGroupNumber, IterMainStageItem)>,
}

impl Iterator for IterMainStageGroup {
    type Item = (
        SpellGroupNumber,
        StageSpellType,
        MainDifficulty,
        NumberedName,
        SpellIdOffset,
    );

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((group_number, item_iter)) = &mut self.cur_item {
            if let Some((difficulty, id_name, offset)) = item_iter.next() {
                return Some((*group_number, self.spell_type, difficulty, id_name, offset));
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
struct MainStageGroup {
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
    type Item = (
        SpellGroupNumber,
        StageSpellType,
        MainDifficulty,
        NumberedName,
        SpellIdOffset,
    );
    type IntoIter = IterMainStageGroup;

    fn into_iter(self) -> Self::IntoIter {
        IterMainStageGroup {
            spell_type: self.spell_type,
            items: self.items.into_iter().enumerate(),
            cur_item: None,
        }
    }
}

struct IterExtraStageGroup {
    spell_type: StageSpellType,
    items: Enumerate<syn::punctuated::IntoIter<NumberedName>>,
}

impl Iterator for IterExtraStageGroup {
    type Item = (SpellGroupNumber, StageSpellType, NumberedName);

    fn next(&mut self) -> Option<Self::Item> {
        self.items
            .next()
            .map(|(group_num, item)| (group_num as SpellGroupNumber, self.spell_type, item))
    }
}

#[derive(Clone)]
struct ExtraStageGroup {
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
    type Item = (SpellGroupNumber, StageSpellType, NumberedName);
    type IntoIter = IterExtraStageGroup;

    fn into_iter(self) -> Self::IntoIter {
        IterExtraStageGroup {
            spell_type: self.spell_type,
            items: self.items.into_iter().enumerate(),
        }
    }
}

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
            Self::Main { stage, inner } => {
                inner
                    .next()
                    .map(|(group_num, spell_type, difficulty, id_name, offset)| {
                        let location = SpellLocation::Main {
                            stage: *stage,
                            difficulty,
                            spell_type,
                        };

                        SpellEntry::new(location, id_name.spell_id, id_name.name, offset, group_num)
                    })
            }
            Self::Extra { stage, inner } => inner.next().map(|(group_num, spell_type, id_name)| {
                let location = SpellLocation::Extra {
                    stage: *stage,
                    spell_type,
                };

                SpellEntry::new(location, id_name.spell_id, id_name.name, 0, group_num)
            }),
            Self::LastWord { lw, inner } => inner.next().map(|(group_num, id_name)| {
                SpellEntry::new(
                    SpellLocation::LastWord(*lw),
                    id_name.spell_id,
                    id_name.name,
                    0,
                    group_num as SpellGroupNumber,
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
