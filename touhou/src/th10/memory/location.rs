use super::state::{Activity, StageState};
use crate::memory::GameLocation;
use crate::th10::{Stage, Touhou10};
use crate::types::{SpellCard, SpellType};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "type", content = "spell")]

pub enum Section {
    Stage,
    Midboss(Option<SpellCard<Touhou10>>),
    Boss(Option<SpellCard<Touhou10>>),
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Location {
    stage: Stage,
    section: Section,
}

impl Location {
    pub fn resolve(state: &StageState) -> Option<Self> {
        let stage = state.stage();
        let section = match state.activity() {
            Activity::StageSection => Section::Stage,
            Activity::Midboss(boss) => Section::Midboss(boss.spell()),
            Activity::Boss(boss) => Section::Boss(boss.spell()),
            Activity::PostDialogue | Activity::StageDialogue => return None,
        };

        Some(Self { stage, section })
    }
}

impl GameLocation<Touhou10> for Location {
    fn name(&self) -> &'static str {
        match self.section {
            Section::Stage => self.stage.name(),
            Section::Midboss(Some(spell)) | Section::Boss(Some(spell)) => spell.name(),
            Section::Midboss(None) => match self.stage {
                Stage::One => "Stage 1 Midboss",
                Stage::Two => "Stage 2 Midboss",
                Stage::Three => "Stage 3 Midboss",
                Stage::Four => "Stage 4 Midboss",
                Stage::Five => "Stage 5 Midboss",
                Stage::Six => "Stage 6 Midboss",
                Stage::Extra => "Extra Midboss",
            },
            Section::Boss(None) => match self.stage {
                Stage::One => "Stage 1 Boss",
                Stage::Two => "Stage 2 Boss",
                Stage::Three => "Stage 3 Boss",
                Stage::Four => "Stage 4 Boss",
                Stage::Five => "Stage 5 Boss",
                Stage::Six => "Stage 6 Boss",
                Stage::Extra => "Extra Boss",
            },
        }
    }

    fn index(&self) -> u64 {
        let stage_bits: u64 = self.stage().into();
        let (section_bits, spell_bits) = match self.section {
            Section::Stage => (0u64, 0),
            Section::Midboss(None) => (1, 0),
            Section::Midboss(Some(spell)) => (2, spell.id() as u64),
            Section::Boss(None) => (3, 0),
            Section::Boss(Some(spell)) => (4, spell.id() as u64),
        };

        (stage_bits << 19) | (section_bits << 16) | spell_bits
    }

    fn stage(&self) -> Stage {
        self.stage
    }

    fn spell(&self) -> Option<SpellCard<Touhou10>> {
        match self.section {
            Section::Midboss(spell) | Section::Boss(spell) => spell,
            Section::Stage => None,
        }
    }

    fn is_end(&self) -> bool {
        false
    }

    fn is_boss_start(&self) -> bool {
        self.spell()
            .as_ref()
            .map(SpellCard::info)
            .is_some_and(|spell| {
                spell.spell_type != SpellType::Midboss && spell.sequence_number == 0
            })
    }

    fn from_spell(spell: SpellCard<Touhou10>) -> Option<Self> {
        Some(Self {
            stage: spell.stage().unwrap(),
            section: if spell.info().spell_type == SpellType::Midboss {
                Section::Midboss(Some(spell))
            } else {
                Section::Boss(Some(spell))
            },
        })
    }
}
