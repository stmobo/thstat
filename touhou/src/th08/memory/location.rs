use std::fmt::{Display, Write};

use serde::{Deserialize, Serialize};

use super::state::StageState;
use crate::th08::{self, Stage, Touhou8};
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
    MidbossSpell { seq: u32, spell: SpellCard<Touhou8> }, // sequence number, ID
    SecondHalf { seq: u32 },
    BossNonspell { seq: u32 },
    BossSpell { seq: u32, spell: SpellCard<Touhou8> },
    BossLastSpell { seq: u32, spell: SpellCard<Touhou8> },
}
