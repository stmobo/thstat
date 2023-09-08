use std::fmt::{Display, Write};

use serde::{Deserialize, Serialize};

use super::state::StageState;
use crate::th08::{self, Stage, Touhou8};
use crate::types::SpellCard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StageSection {
    Start,
    Stage { seq: u32 },
    Nonspell { midboss: bool, seq: u32 },
    Spell { spell: SpellCard<Touhou8> }
}

// impl StageSection {
//     pub fn new(state: &StageState) -> Option<Self> {
//         let boss_state = state.boss_state();
//         let active_spell = boss_state.and_then(|boss| boss.active_spell()).map(|spell| SpellCard::new((spell.id() + 1).try_into().unwrap()));

//         if let Some(spell) = active_spell {
//             Some(StageSection::Spell { spell })
//         } else {
//             Some(match state.stage() {
//                 Stage::One => match state.frame() {
//                     0..=339 => StageSection::Start,
//                     340..=2874 => StageSection::Stage { seq: 0 },
//                     2875..=2934 => StageSection::Stage { seq: 1 },
//                     2935..=4174 => StageSection::Nonspell { midboss: true, seq: 0 },
//                     _ => if let Some(bars) = boss_state.map(|boss| boss.remaining_lifebars()) {

//                     }
//                 }
//             })
//         }

//     }
// }
