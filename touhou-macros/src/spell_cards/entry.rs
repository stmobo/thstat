use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Result, Token};

use super::spell_data::{
    Difficulty, ExtraStage, MainDifficulty, MainStage, SpellLocation, SpellType, Stage,
    StageSpellType,
};

pub type SpellIdOffset = u32;
pub type SpellGroupNumber = u32;

mod extra_stage;
mod main_stage;
mod stage_set;

use extra_stage::ExtraStageGroup;
use main_stage::MainStageGroup;
pub use stage_set::{IterStageSet, StageSet};

#[derive(Clone)]
pub struct NumberedName {
    _hash: Option<Token![#]>,
    spell_id: LitInt,
    name: LitStr,
}

impl NumberedName {
    pub fn spell_id(&self) -> &LitInt {
        &self.spell_id
    }

    pub fn name(&self) -> &LitStr {
        &self.name
    }
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
struct PartialEntry<A, B, C, D, E> {
    id_name: NumberedName,
    difficulty: A,
    offset: B,
    group_number: C,
    spell_type: D,
    allow_overlap: E,
}

impl PartialEntry<(), (), (), (), ()> {
    pub fn new(id_name: NumberedName) -> Self {
        Self {
            id_name,
            difficulty: (),
            offset: (),
            group_number: (),
            spell_type: (),
            allow_overlap: (),
        }
    }
}

impl<A, B, C, D, E> PartialEntry<A, B, C, D, E> {
    pub fn attach_main_difficulty(
        self,
        difficulty: MainDifficulty,
    ) -> PartialEntry<MainDifficulty, B, C, D, E> {
        PartialEntry {
            difficulty,
            id_name: self.id_name,
            offset: self.offset,
            group_number: self.group_number,
            spell_type: self.spell_type,
            allow_overlap: self.allow_overlap,
        }
    }

    pub fn attach_offset(self, offset: SpellIdOffset) -> PartialEntry<A, SpellIdOffset, C, D, E> {
        PartialEntry {
            difficulty: self.difficulty,
            id_name: self.id_name,
            offset,
            group_number: self.group_number,
            spell_type: self.spell_type,
            allow_overlap: self.allow_overlap,
        }
    }

    pub fn attach_group_number(
        self,
        group_number: SpellGroupNumber,
    ) -> PartialEntry<A, B, SpellGroupNumber, D, E> {
        PartialEntry {
            difficulty: self.difficulty,
            id_name: self.id_name,
            offset: self.offset,
            group_number,
            spell_type: self.spell_type,
            allow_overlap: self.allow_overlap,
        }
    }

    pub fn attach_spell_type(
        self,
        spell_type: StageSpellType,
    ) -> PartialEntry<A, B, C, StageSpellType, E> {
        PartialEntry {
            difficulty: self.difficulty,
            id_name: self.id_name,
            offset: self.offset,
            group_number: self.group_number,
            spell_type,
            allow_overlap: self.allow_overlap,
        }
    }

    pub fn set_allow_overlap(self, allow_overlap: bool) -> PartialEntry<A, B, C, D, bool> {
        PartialEntry {
            difficulty: self.difficulty,
            id_name: self.id_name,
            offset: self.offset,
            group_number: self.group_number,
            spell_type: self.spell_type,
            allow_overlap,
        }
    }
}

impl PartialEntry<(), (), SpellGroupNumber, StageSpellType, ()> {
    pub fn into_extra_stage_entry(self, stage: ExtraStage) -> SpellEntry {
        let location = SpellLocation::Extra {
            stage,
            spell_type: self.spell_type,
        };

        SpellEntry::new(
            location,
            self.id_name.spell_id.clone(),
            self.id_name.name.clone(),
            0,
            self.group_number,
            false,
        )
    }
}

impl PartialEntry<MainDifficulty, SpellIdOffset, SpellGroupNumber, StageSpellType, bool> {
    pub fn into_main_stage_entry(self, stage: MainStage) -> SpellEntry {
        let location = SpellLocation::Main {
            stage,
            difficulty: self.difficulty,
            spell_type: self.spell_type,
        };

        SpellEntry::new(
            location,
            self.id_name.spell_id().clone(),
            self.id_name.name().clone(),
            self.offset,
            self.group_number,
            self.allow_overlap,
        )
    }
}

#[derive(Clone)]
pub struct SpellEntry {
    location: SpellLocation,
    group_num: SpellGroupNumber,
    id: (LitInt, u32),
    name: (LitStr, String),
    allow_overlap: bool,
}

impl SpellEntry {
    pub fn new(
        location: SpellLocation,
        id: LitInt,
        name: LitStr,
        offset: SpellIdOffset,
        group_num: SpellGroupNumber,
        allow_overlap: bool,
    ) -> Self {
        let parsed_id: u32 = id.base10_parse().unwrap();
        let parsed_name = name.value();

        Self {
            location,
            id: (id, parsed_id + offset),
            name: (name, parsed_name),
            group_num,
            allow_overlap,
        }
    }

    pub fn id(&self) -> u32 {
        self.id.1
    }

    pub fn name(&self) -> &str {
        &self.name.1
    }

    pub fn id_span(&self) -> &LitInt {
        &self.id.0
    }

    pub fn group_number(&self) -> u32 {
        self.group_num
    }

    pub fn location(&self) -> SpellLocation {
        self.location
    }

    pub fn allow_overlap(&self) -> bool {
        self.allow_overlap
    }

    pub fn spell_def_tokens(&self, game_ident: &Ident) -> TokenStream {
        let difficulty: Difficulty = self.location.into();
        let stage: Stage = self.location.into();
        let spell_type: SpellType = self.location.into();
        let group_num = self.group_num;

        let name = &self.name.1;
        quote! {
            crate::types::SpellCardInfo::<#game_ident> {
                name: #name,
                difficulty: #difficulty,
                stage: #stage,
                spell_type: #spell_type,
                sequence_number: #group_num
            }
        }
    }
}
