
use std::fmt::{Debug, Display};
use std::hash::Hash;

use std::ops::Deref;
use std::str;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{impl_wrapper_traits, Difficulty, Game, GameId, SpellCardId, Stage};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpellCardInfo {
    name: &'static str,
    difficulty: Difficulty,
    stage: Stage,
    is_midboss: bool,
}

impl SpellCardInfo {
    pub(crate) const fn new(
        name: &'static str,
        difficulty: Difficulty,
        stage: Stage,
        is_midboss: bool,
    ) -> Self {
        Self {
            name,
            difficulty,
            stage,
            is_midboss,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub const fn stage(&self) -> Stage {
        self.stage
    }

    pub const fn is_midboss(&self) -> bool {
        self.is_midboss
    }
}

#[repr(transparent)]
pub struct SpellCard<G: Game>(G::SpellID);

impl<G: Game> AsRef<G::SpellID> for SpellCard<G> {
    fn as_ref(&self) -> &G::SpellID {
        &self.0
    }
}

impl<G: Game> Deref for SpellCard<G> {
    type Target = G::SpellID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G: Game> SpellCard<G> {
    pub const fn new(card_id: G::SpellID) -> Self {
        Self(card_id)
    }

    pub fn unwrap(self) -> G::SpellID {
        self.0
    }

    pub fn id(&self) -> u32 {
        self.0.raw_id()
    }

    pub fn info(&self) -> &'static SpellCardInfo {
        self.0.card_info()
    }

    pub fn name(&self) -> &'static str {
        self.info().name
    }

    pub fn difficulty(&self) -> Difficulty {
        self.info().difficulty
    }

    pub fn stage(&self) -> Stage {
        self.info().stage
    }

    pub fn is_midboss(&self) -> bool {
        self.info().is_midboss
    }
}

impl<G: Game> Debug for SpellCard<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpellCard<{}>({:?} : {})",
            self.0.game_id().abbreviation(),
            self.0,
            self.name()
        )
    }
}

impl<G: Game> Display for SpellCard<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} #{}: {}",
            self.0.game_id().abbreviation(),
            self.0.raw_id(),
            self.name()
        )
    }
}

impl_wrapper_traits!(SpellCard, u32, G::SpellID);

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidCardId {
    #[error("Invalid card ID {1} for {0} (valid values are 1..={2})")]
    InvalidCard(GameId, u32, u32),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
}

macro_rules! spellcard_data {
    {
        n: $n:literal,
        $(
            $stage:ident : {
                $(midboss: {
                    $([
                        $((
                            $mid_easy_name:literal,
                            $mid_normal_name:literal
                        ),)?
                        $mid_hard_name:literal,
                        $mid_luna_name:literal
                    ]),+
                },)?
                boss: {
                    $([
                        $boss_easy_name:literal,
                        $boss_normal_name:literal,
                        $boss_hard_name:literal,
                        $boss_luna_name:literal
                    ]),+
                }
            },
        )+
        {
            midboss: [
                $($extra_mid_name:literal),+
            ],
            boss: [
                $($extra_boss_name:literal),+
            ]
        }
        $(
            , {
                midboss: [
                    $($phantasm_mid_name:literal),+
                ],
                boss: [
                    $($phantasm_boss_name:literal),+
                ]
            }
        )?
    } => {
        pub(crate) const SPELL_CARDS: &[$crate::types::SpellCardInfo] = &[
            $(
                $($(
                    $(
                        $crate::types::SpellCardInfo::new($mid_easy_name, $crate::types::Difficulty::Easy, $stage, true),
                        $crate::types::SpellCardInfo::new($mid_normal_name, $crate::types::Difficulty::Normal, $stage, true),
                    )?
                    $crate::types::SpellCardInfo::new($mid_hard_name, $crate::types::Difficulty::Hard, $stage, true),
                    $crate::types::SpellCardInfo::new($mid_luna_name, $crate::types::Difficulty::Lunatic, $stage, true),
                )+)?
                $(
                    $crate::types::SpellCardInfo::new($boss_easy_name, $crate::types::Difficulty::Easy, $stage, false),
                    $crate::types::SpellCardInfo::new($boss_normal_name, $crate::types::Difficulty::Normal, $stage, false),
                    $crate::types::SpellCardInfo::new($boss_hard_name, $crate::types::Difficulty::Hard, $stage, false),
                    $crate::types::SpellCardInfo::new($boss_luna_name, $crate::types::Difficulty::Lunatic, $stage, false),
                )+
            )+
            $(
                $crate::types::SpellCardInfo::new($extra_mid_name, $crate::types::Difficulty::Extra, $crate::types::Stage::Extra, true),
            )+
            $(
                $crate::types::SpellCardInfo::new($extra_boss_name, $crate::types::Difficulty::Extra, $crate::types::Stage::Extra, false),
            )+
            $(
                $(
                    $crate::types::SpellCardInfo::new($phantasm_mid_name, $crate::types::Difficulty::Phantasm, $crate::types::Stage::Phantasm, true),
                )+
                $(
                    $crate::types::SpellCardInfo::new($phantasm_boss_name, $crate::types::Difficulty::Phantasm, $crate::types::Stage::Phantasm, false),
                )+
            )?
        ];
    };
}

pub(crate) use spellcard_data;
