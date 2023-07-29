use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::str;

use serde::{Deserialize, Serialize};

use super::{impl_wrapper_traits, Difficulty, Game, IterableEnum, Stage};

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
pub struct SpellCard<G: Game>(u16, PhantomData<G>);

impl<G: Game> SpellCard<G> {
    pub const fn new(card_id: u16) -> Result<Self, InvalidCardId<G>> {
        if (card_id > 0) && ((card_id as usize) <= G::CARD_INFO.len()) {
            Ok(Self(card_id, PhantomData))
        } else {
            Err(InvalidCardId(card_id, PhantomData))
        }
    }

    pub const fn id(&self) -> u16 {
        self.0
    }

    pub const fn info(&self) -> &'static SpellCardInfo {
        &G::CARD_INFO[(self.0 - 1) as usize]
    }

    pub const fn name(&self) -> &'static str {
        self.info().name
    }

    pub const fn difficulty(&self) -> Difficulty {
        self.info().difficulty
    }

    pub const fn stage(&self) -> Stage {
        self.info().stage
    }

    pub const fn is_midboss(&self) -> bool {
        self.info().is_midboss
    }
}

impl<G: Game> Debug for SpellCard<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpellCard<{}>({} : {})",
            G::GAME_ID.abbreviation(),
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
            G::GAME_ID.abbreviation(),
            self.0,
            self.name()
        )
    }
}

impl_wrapper_traits!(SpellCard, u16);

impl<G: Game> TryFrom<u16> for SpellCard<G> {
    type Error = InvalidCardId<G>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

pub struct SpellCardIter<G: Game>(u16, PhantomData<G>);

impl<G: Game> Debug for SpellCardIter<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpellCardIter<{}>({})",
            G::GAME_ID.abbreviation(),
            self.0
        )
    }
}

impl<G: Game> Iterator for SpellCardIter<G> {
    type Item = SpellCard<G>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.0 as usize) < G::CARD_INFO.len() {
            let v = self.0;
            self.0 += 1;
            Some(SpellCard(v + 1, PhantomData))
        } else {
            None
        }
    }
}

impl<G: Game> IterableEnum for SpellCard<G> {
    type EnumIter = SpellCardIter<G>;

    fn iter_all() -> Self::EnumIter {
        SpellCardIter(0, PhantomData)
    }
}

pub struct InvalidCardId<G: Game>(u16, PhantomData<G>);

impl<G: Game> Debug for InvalidCardId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InvalidCardId<{}>({}, valid range 1..={})",
            G::GAME_ID.abbreviation(),
            self.0,
            G::CARD_INFO.len() + 1
        )
    }
}

impl<G: Game> Display for InvalidCardId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid card ID {} for {} (valid range 1..={})",
            self.0,
            G::GAME_ID.abbreviation(),
            G::CARD_INFO.len() + 1
        )
    }
}

impl<G: Game> Error for InvalidCardId<G> {}

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
