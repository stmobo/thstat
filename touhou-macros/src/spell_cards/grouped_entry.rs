use std::collections::hash_map::Entry;
use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, token, Ident, LitInt, LitStr, Result, Token};

use super::spell_data::{kw, MainDifficulty, MainStage, SpellEntry, SpellLocation, Stage};

#[derive(Clone)]
pub struct NumberedName {
    hash: Option<Token![#]>,
    spell_id: LitInt,
    name: LitStr,
}

impl NumberedName {
    pub fn id(&self) -> Result<u16> {
        self.spell_id.base10_parse()
    }

    pub fn name(&self) -> String {
        self.name.value()
    }
}

impl Parse for NumberedName {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            hash: input.parse()?,
            spell_id: input.parse()?,
            name: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub struct DifficultyList(Punctuated<MainDifficulty, Token![|]>);

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

pub enum ExpandGroupEntry {
    Main {
        difficulties: syn::punctuated::IntoIter<MainDifficulty>,
        stage: MainStage,
        midboss: Option<kw::Midboss>,
        id_name: NumberedName,
        offset: u32,
    },
    Single(Option<(SpellLocation, NumberedName)>),
}

impl Iterator for ExpandGroupEntry {
    type Item = SpellEntry;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Main {
                difficulties,
                stage,
                midboss,
                id_name,
                offset,
            } => {
                if let Some(difficulty) = difficulties.next() {
                    let location = stage.clone().into_spell_location(difficulty, *midboss);
                    let ret = SpellEntry::new(
                        location,
                        id_name.spell_id.clone(),
                        id_name.name.clone(),
                        *offset,
                    );
                    *offset += 1;
                    Some(ret)
                } else {
                    None
                }
            }
            Self::Single(item) => item.take().map(|(location, id_name)| {
                SpellEntry::new(location, id_name.spell_id, id_name.name, 0)
            }),
        }
    }
}

#[derive(Clone)]
pub enum GroupEntry {
    Main {
        difficulties: DifficultyList,
        colon: Token![:],
        id_name: NumberedName,
        stage: MainStage,
        midboss: Option<(kw::Midboss, Token![:])>,
    },
    Extra {
        id_name: NumberedName,
        extra_token: kw::Extra,
        midboss: Option<(kw::Midboss, Token![:])>,
    },
    Phantasm {
        id_name: NumberedName,
        phantasm_token: kw::Phantasm,
        midboss: Option<(kw::Midboss, Token![:])>,
    },
    LastWord {
        id_name: NumberedName,
        lw_token: kw::LastWord,
    },
}

impl GroupEntry {
    fn parse(
        stage: Stage,
        midboss: Option<(kw::Midboss, Token![:])>,
        input: ParseStream,
    ) -> Result<Self> {
        match stage.into_main_stage() {
            Ok(stage) => Ok(Self::Main {
                difficulties: input.parse()?,
                colon: input.parse()?,
                id_name: input.parse()?,
                stage,
                midboss,
            }),
            Err(Stage::Extra(extra_token)) => Ok(Self::Extra {
                id_name: input.parse()?,
                extra_token,
                midboss,
            }),
            Err(Stage::Phantasm(phantasm_token)) => Ok(Self::Phantasm {
                id_name: input.parse()?,
                phantasm_token,
                midboss,
            }),
            Err(Stage::LastWord(lw_token)) => Ok(Self::LastWord {
                id_name: input.parse()?,
                lw_token,
            }),
            Err(_) => unreachable!(),
        }
    }

    fn parse_terminated(
        stage: &Stage,
        midboss: &Option<(kw::Midboss, Token![:])>,
        input: ParseStream,
    ) -> Result<Punctuated<Self, Token![,]>> {
        let mut ret = Punctuated::new();

        while !input.is_empty() {
            ret.push_value(Self::parse(stage.clone(), *midboss, input)?);

            if !input.is_empty() {
                ret.push_punct(input.parse()?);
            }
        }

        Ok(ret)
    }

    pub fn into_spell_entries(self) -> ExpandGroupEntry {
        match self {
            Self::Main {
                difficulties,
                id_name,
                stage,
                midboss,
                ..
            } => {
                let midboss = midboss.map(|pair| pair.0);
                ExpandGroupEntry::Main {
                    difficulties: difficulties.0.into_iter(),
                    stage,
                    midboss,
                    id_name,
                    offset: 0,
                }
            }
            Self::Extra {
                id_name,
                extra_token,
                midboss,
            } => ExpandGroupEntry::Single(Some((
                SpellLocation::Extra {
                    stage_token: extra_token,
                    midboss: midboss.map(|pair| pair.0),
                },
                id_name,
            ))),
            Self::Phantasm {
                id_name,
                phantasm_token,
                midboss,
            } => ExpandGroupEntry::Single(Some((
                SpellLocation::Phantasm {
                    stage_token: phantasm_token,
                    midboss: midboss.map(|pair| pair.0),
                },
                id_name,
            ))),
            Self::LastWord { id_name, lw_token } => ExpandGroupEntry::Single(Some((
                SpellLocation::LastWord {
                    stage_token: lw_token,
                },
                id_name,
            ))),
        }
    }
}

pub struct SpellGroup {
    midboss: Option<(kw::Midboss, Token![:])>,
    bracket: token::Bracket,
    entries: Punctuated<GroupEntry, Token![,]>,
}

impl SpellGroup {
    fn parse(stage: &Stage, input: ParseStream) -> Result<Self> {
        let lookahead1 = input.lookahead1();

        let midboss = if stage.can_have_midboss() && lookahead1.peek(kw::Midboss) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        let content;
        Ok(Self {
            midboss: midboss.clone(),
            bracket: bracketed!(content in input),
            entries: GroupEntry::parse_terminated(stage, &midboss, &content)?,
        })
    }

    fn parse_terminated(stage: &Stage, input: ParseStream) -> Result<Punctuated<Self, Token![,]>> {
        let mut ret = Punctuated::new();

        while !input.is_empty() {
            ret.push_value(Self::parse(stage, input)?);

            if !input.is_empty() {
                ret.push_punct(input.parse()?);
            }
        }

        Ok(ret)
    }

    pub fn into_spell_entries(self) -> impl Iterator<Item = SpellEntry> {
        self.entries
            .into_iter()
            .flat_map(GroupEntry::into_spell_entries)
    }
}

pub struct StageSet {
    stage: Stage,
    colon: Token![:],
    brace: token::Brace,
    groups: Punctuated<SpellGroup, Token![,]>,
}

impl StageSet {
    pub fn into_spell_entries(self) -> impl Iterator<Item = SpellEntry> {
        self.groups
            .into_iter()
            .flat_map(|group| group.into_spell_entries())
    }
}

impl Parse for StageSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let stage: Stage = input.parse()?;
        let content;

        Ok(Self {
            stage: stage.clone(),
            colon: input.parse()?,
            brace: braced!(content in input),
            groups: SpellGroup::parse_terminated(&stage, &content)?,
        })
    }
}
