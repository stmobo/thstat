use std::fmt::{Display, Write};

use serde::{Deserialize, Serialize};

use super::state::StageState;
use crate::th07::{self, Stage, Touhou7};
use crate::types::SpellCard;

macro_rules! convert_to_spellcard {
    ($x:expr) => {
        SpellCard::new(($x + 1).try_into().unwrap())
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StageSection {
    Start,
    FirstHalf { seq: u32 },
    MidbossNonspell { seq: u32 },
    MidbossSpell { seq: u32, spell: SpellCard<Touhou7> }, // sequence number, ID
    SecondHalf { seq: u32 },
    PreBoss,
    BossNonspell { seq: u32 },
    BossSpell { seq: u32, spell: SpellCard<Touhou7> },
}

impl StageSection {
    pub fn new(state: &StageState) -> Option<Self> {
        Some(match state.stage() {
            Stage::One => match state.ecl_time() {
                0..=539 => StageSection::Start,
                540..=1340 => StageSection::FirstHalf { seq: 0 },
                1341..=2655 => StageSection::FirstHalf { seq: 1 },
                2656..=3106 => {
                    if let Some(boss) = &state.boss_state() {
                        if let Some(spell) = &boss.active_spell() {
                            StageSection::MidbossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(spell.0),
                            }
                        } else {
                            StageSection::MidbossNonspell { seq: 0 }
                        }
                    } else {
                        StageSection::SecondHalf { seq: 0 }
                    }
                }
                3107..=5041 => StageSection::SecondHalf { seq: 0 },
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 2..=5) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 6..=9) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                1 => StageSection::BossNonspell { seq: 0 },
                                0 => StageSection::BossNonspell { seq: 1 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Two => match state.ecl_time() {
                0..=389 => StageSection::Start,
                390..=2825 => StageSection::FirstHalf { seq: 0 },
                2826..=3365 => {
                    if let Some(boss) = &state.boss_state() {
                        if let Some(spell) = &boss.active_spell() {
                            StageSection::MidbossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(spell.0),
                            }
                        } else {
                            StageSection::MidbossNonspell { seq: 0 }
                        }
                    } else {
                        StageSection::SecondHalf { seq: 0 }
                    }
                }
                3366..=7646 => StageSection::SecondHalf { seq: 0 },
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 14..=17) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 18..=21) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 22..=25) => StageSection::BossSpell {
                                seq: 2,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                1 => StageSection::BossNonspell { seq: 0 },
                                0 => StageSection::BossNonspell { seq: 1 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Three => match state.ecl_time() {
                0..=389 => StageSection::Start,
                390..=820 => StageSection::FirstHalf { seq: 0 },
                821..=1804 => {
                    if state.boss_state().is_some() {
                        StageSection::MidbossNonspell { seq: 0 }
                    } else {
                        StageSection::FirstHalf { seq: 1 }
                    }
                }
                1805..=1857 => StageSection::SecondHalf { seq: 0 },
                1858..=3392 => {
                    if let Some(boss) = &state.boss_state() {
                        if let Some(spell) = &boss.active_spell() {
                            StageSection::MidbossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(spell.0),
                            }
                        } else {
                            StageSection::MidbossNonspell { seq: 1 }
                        }
                    } else {
                        StageSection::SecondHalf { seq: 0 }
                    }
                }
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 28..=31) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 32..=35) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 36..=39) => StageSection::BossSpell {
                                seq: 2,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 40..=43) => StageSection::BossSpell {
                                seq: 3,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                2 => StageSection::BossNonspell { seq: 0 },
                                1 => StageSection::BossNonspell { seq: 1 },
                                0 => StageSection::BossNonspell { seq: 2 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Four => match state.ecl_time() {
                0..=79 => StageSection::Start,
                80..=1947 => StageSection::FirstHalf { seq: 0 },
                1948..=3026 => StageSection::FirstHalf { seq: 1 },
                3028..=4286 => StageSection::FirstHalf { seq: 2 },
                4288..=7121 => StageSection::FirstHalf { seq: 3 },
                7122..=7963 => StageSection::MidbossNonspell { seq: 0 },
                7964..=10135 => {
                    if state.boss_state().is_some() {
                        StageSection::MidbossNonspell { seq: 0 }
                    } else {
                        StageSection::SecondHalf { seq: 0 }
                    }
                }
                10136..=11395 => StageSection::SecondHalf { seq: 1 },
                11396..=13165 => StageSection::SecondHalf { seq: 2 },
                13166..=14825 => StageSection::SecondHalf { seq: 3 },
                14826..=15199 => {
                    if state.boss_state().is_some() {
                        StageSection::BossNonspell { seq: 0 }
                    } else {
                        StageSection::PreBoss
                    }
                }
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 44..=47) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 48..=51) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 52..=55) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 56..=59) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 60..=63) => StageSection::BossSpell {
                                seq: 2,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 65..=68) => StageSection::BossSpell {
                                seq: 3,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                3 => StageSection::BossNonspell { seq: 0 },
                                2 => StageSection::BossNonspell { seq: 1 },
                                1 => StageSection::BossNonspell { seq: 2 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Five => match state.ecl_time() {
                0..=439 => StageSection::Start,
                440..=839 => StageSection::FirstHalf { seq: 0 },
                840..=2549 => StageSection::FirstHalf { seq: 1 },
                2550..=4819 => StageSection::FirstHalf { seq: 2 },
                4820..=4882 => {
                    if let Some(boss) = &state.boss_state() {
                        if let Some(spell) = &boss.active_spell() {
                            StageSection::MidbossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(spell.0),
                            }
                        } else {
                            StageSection::MidbossNonspell { seq: 0 }
                        }
                    } else {
                        StageSection::SecondHalf { seq: 0 }
                    }
                }
                4883..=6112 => StageSection::SecondHalf { seq: 0 },
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 72..=75) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 76..=79) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 80..=83) => StageSection::BossSpell {
                                seq: 2,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 84..=87) => StageSection::BossSpell {
                                seq: 3,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                2 => StageSection::BossNonspell { seq: 0 },
                                1 => StageSection::BossNonspell { seq: 1 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Six => match state.ecl_time() {
                0..=659 => StageSection::Start,
                660..=1179 => StageSection::FirstHalf { seq: 0 },
                1180..=1913 => StageSection::FirstHalf { seq: 1 },
                1914..=2517 => {
                    if let Some(boss) = &state.boss_state() {
                        if let Some(spell) = &boss.active_spell() {
                            StageSection::MidbossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(spell.0),
                            }
                        } else {
                            StageSection::MidbossNonspell { seq: 0 }
                        }
                    } else {
                        StageSection::PreBoss
                    }
                }
                _ => {
                    if let Some(boss) = &state.boss_state() {
                        match boss.active_spell().map(|x| x.0) {
                            Some(id @ 92..=95) => StageSection::BossSpell {
                                seq: 0,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 96..=99) => StageSection::BossSpell {
                                seq: 1,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 100..=103) => StageSection::BossSpell {
                                seq: 2,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 104..=107) => StageSection::BossSpell {
                                seq: 3,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 108..=111) => StageSection::BossSpell {
                                seq: 4,
                                spell: convert_to_spellcard!(id),
                            },
                            Some(id @ 112..=115) => StageSection::BossSpell {
                                seq: 5,
                                spell: convert_to_spellcard!(id),
                            },
                            None => match boss.remaining_lifebars() {
                                4 => StageSection::BossNonspell { seq: 0 },
                                3 => StageSection::BossNonspell { seq: 1 },
                                2 => StageSection::BossNonspell { seq: 2 },
                                1 => StageSection::BossNonspell { seq: 3 },
                                _ => return None,
                            },
                            _ => return None,
                        }
                    } else {
                        return None;
                    }
                }
            },
            Stage::Extra | Stage::Phantasm => return None,
        })
    }
}

impl Display for StageSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => f.write_str("Start"),
            Self::FirstHalf { seq } => write!(f, "First Half {}", *seq + 1),
            Self::MidbossNonspell { seq } => write!(f, "Midboss Nonspell {}", *seq + 1),
            Self::MidbossSpell { seq, spell } => {
                write!(
                    f,
                    "Midboss Spell {} (#{:03} {})",
                    *seq + 1,
                    spell.id(),
                    spell.name()
                )
            }
            Self::SecondHalf { seq } => write!(f, "Second Half {}", *seq + 1),
            Self::PreBoss => f.write_str("Pre-Boss"),
            Self::BossNonspell { seq } => write!(f, "Boss Nonspell {}", *seq + 1),
            Self::BossSpell { seq, spell } => write!(
                f,
                "Boss Spell {} (#{:03} {})",
                *seq + 1,
                spell.id(),
                spell.name()
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StageLocation {
    stage: th07::Stage,
    section: StageSection,
}

impl StageLocation {
    pub fn new(state: &StageState) -> Option<Self> {
        StageSection::new(state).map(|section| Self {
            stage: state.stage(),
            section,
        })
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn section(&self) -> StageSection {
        self.section
    }

    pub fn is_end_spell(&self) -> bool {
        matches!(
            (self.stage, self.section),
            (Stage::One, StageSection::BossSpell { seq: 1, .. })
                | (Stage::Two, StageSection::BossSpell { seq: 2, .. })
                | (
                    Stage::Three | Stage::Four | Stage::Five,
                    StageSection::BossSpell { seq: 3, .. }
                )
                | (Stage::Six, StageSection::BossSpell { seq: 5, .. })
        )
    }
}

impl Display for StageLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stage.fmt(f)?;

        if self.stage == Stage::Six && self.section == (StageSection::FirstHalf { seq: 1 }) {
            f.write_str(" Spam")
        } else {
            f.write_char(' ')?;
            self.section.fmt(f)
        }
    }
}
