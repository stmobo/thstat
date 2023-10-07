use super::state::Activity;
use super::{BossState, RunState};
use crate::memory::GameLocation;
use crate::th10::{Difficulty, Stage, Touhou10};
use crate::types::{SpellCard, SpellType, Stage as StageWrapper};

macro_rules! nonspell_strings {
    {
        $type_str:literal,
        $($stage:literal :[ $($str:literal),* ]),*
    } => {
        [
            $(
                &[
                    $( concat!($type_str, " Nonspell ", $str) ),*
                ]
            ),*
        ]
    }
}

const MIDBOSS_NONSPELL_STRINGS: [&[&str]; 7] = nonspell_strings!(
    "Midboss",
    "Stage 1": ["1", "2"],
    "Stage 2": [],
    "Stage 3": ["1"],
    "Stage 4": [],
    "Stage 5": ["1"],
    "Stage 6": [],
    "Extra Stage": ["1"]
);

const BOSS_NONSPELL_STRINGS: [&[&str]; 7] = nonspell_strings!(
    "Boss",
    "Stage 1": ["1", "2"],
    "Stage 2": ["1", "2"],
    "Stage 3": ["1", "2", "3"],
    "Stage 4": ["1", "2", "3"],
    "Stage 5": ["1", "2", "3"],
    "Stage 6": ["1", "2", "3", "4"],
    "Extra Stage": ["1", "2", "3", "4", "5", "6", "7", "8"]
);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct BossSection {
    seq: u32,
    spell: Option<SpellCard<Touhou10>>,
}

impl BossSection {
    fn from_spell(spell: SpellCard<Touhou10>) -> Self {
        Self {
            seq: spell.sequence_number,
            spell: Some(spell),
        }
    }

    fn resolve_boss(stage: Stage, boss: &BossState) -> Option<Self> {
        boss.spell().map(BossSection::from_spell).or_else(|| {
            let lifebars = boss.remaining_lifebars() as u32;
            match stage {
                Stage::One | Stage::Two => 1u32.checked_sub(lifebars),
                Stage::Three | Stage::Four => 2u32.checked_sub(lifebars),
                Stage::Five => lifebars.checked_sub(1).and_then(|n| 2u32.checked_sub(n)),
                Stage::Six => lifebars.checked_sub(1).and_then(|n| 3u32.checked_sub(n)),
                Stage::Extra => lifebars.checked_sub(3).and_then(|v| 7u32.checked_sub(v)),
            }
            .map(|seq| BossSection { seq, spell: None })
        })
    }

    fn resolve_midboss(stage: Stage, difficulty: Difficulty, boss: &BossState) -> Option<Self> {
        boss.spell().map(BossSection::from_spell).or_else(|| {
            let lifebars = boss.remaining_lifebars() as u32;
            match stage {
                Stage::One => {
                    if difficulty <= Difficulty::Normal {
                        1u32.checked_sub(lifebars)
                    } else {
                        Some(0)
                    }
                }
                Stage::Two | Stage::Four | Stage::Six => None,
                Stage::Three | Stage::Five | Stage::Extra => Some(0),
            }
            .map(|seq| BossSection { seq, spell: None })
        })
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "type", content = "spell")]

pub enum Section {
    Stage,
    Midboss(BossSection),
    Boss(BossSection),
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Location {
    stage: Stage,
    section: Section,
}

impl Location {
    pub fn is_stage_section(&self) -> bool {
        self.section == Section::Stage
    }

    pub fn resolve(state: &RunState) -> Option<Self> {
        let stage_state = state.stage();
        let stage = stage_state.stage().unwrap();
        let difficulty = state.difficulty().unwrap();

        match stage_state.activity() {
            Activity::StageSection => Some(Section::Stage),
            Activity::Midboss(boss) => {
                BossSection::resolve_midboss(stage, difficulty, &boss).map(Section::Midboss)
            }
            Activity::Boss(boss) => BossSection::resolve_boss(stage, &boss).map(Section::Boss),
            Activity::PostDialogue | Activity::StageDialogue => None,
        }
        .map(|section| Self { stage, section })
    }
}

impl GameLocation<Touhou10> for Location {
    fn name(&self) -> &'static str {
        match self.section {
            Section::Stage => self.stage.name(),
            Section::Midboss(boss) => {
                if let Some(spell) = &boss.spell {
                    spell.name
                } else {
                    MIDBOSS_NONSPELL_STRINGS[usize::from(self.stage)]
                        .get(boss.seq as usize)
                        .expect("invalid nonspell sequence number")
                }
            }
            Section::Boss(boss) => {
                if let Some(spell) = &boss.spell {
                    spell.name()
                } else {
                    BOSS_NONSPELL_STRINGS[usize::from(self.stage)]
                        .get(boss.seq as usize)
                        .expect("invalid nonspell sequence number")
                }
            }
        }
    }

    fn index(&self) -> u64 {
        let stage_bits: u64 = self.stage().unwrap().into();
        let (section_bits, spell_bits) = match self.section {
            Section::Stage => (0u64, 0),
            Section::Midboss(boss) => {
                if let Some(spell) = &boss.spell {
                    (2, spell.id() as u64)
                } else {
                    (1, boss.seq as u64)
                }
            }
            Section::Boss(boss) => {
                if let Some(spell) = &boss.spell {
                    (4, spell.id() as u64)
                } else {
                    (3, boss.seq as u64)
                }
            }
        };

        (stage_bits << 19) | (section_bits << 16) | spell_bits
    }

    fn stage(&self) -> StageWrapper<Touhou10> {
        StageWrapper::new(self.stage)
    }

    fn spell(&self) -> Option<SpellCard<Touhou10>> {
        match self.section {
            Section::Midboss(boss) | Section::Boss(boss) => boss.spell,
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
        let stage = spell.stage.unwrap();
        let section = if spell.spell_type == SpellType::Midboss {
            Section::Midboss(BossSection::from_spell(spell))
        } else {
            Section::Boss(BossSection::from_spell(spell))
        };

        Some(Self { stage, section })
    }
}
