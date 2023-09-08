use std::fmt::Display;
use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};
#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, System, SystemExt};
use touhou_macros::NumericEnum;

pub mod score;
pub mod spellcards;

#[cfg(feature = "memory")]
pub mod memory;

pub use score::ScoreFile;
use spellcards::SPELL_CARDS;

use crate::types::{
    Game, GameId, GameValue, InvalidCardId, InvalidDifficultyId, InvalidShotType, InvalidStageId,
    ShotType as WrappedShot, SpellCardInfo,
};

fn invalid_shot_type(value: u64) -> InvalidShotType {
    InvalidShotType::InvalidShotId(value as u16, 0, 5)
}

#[derive(Debug, NumericEnum)]
#[error_type = "InvalidShotType"]
#[convert_error = "invalid_shot_type"]
pub enum ShotType {
    #[name = "Reimu & Yukari"]
    BarrierTeam = 0,
    #[name = "Marisa & Alice"]
    MagicTeam = 1,
    #[name = "Sakuya & Remilia"]
    ScarletTeam = 2,
    #[name = "Youmu & Yuyuko"]
    GhostTeam = 3,
    Reimu = 4,
    Yukari = 5,
    Marisa = 6,
    Alice = 7,
    Sakuya = 8,
    Remilia = 9,
    Youmu = 10,
    Yuyuko = 11,
}

impl GameValue for ShotType {
    type RawValue = u16;
    type ConversionError = InvalidShotType;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidShotType::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellId(NonZeroU16);

impl SpellId {
    pub fn card_info(&self) -> &'static SpellCardInfo<Touhou8> {
        &SPELL_CARDS[(self.0.get() - 1) as usize]
    }
}

impl From<SpellId> for u32 {
    fn from(value: SpellId) -> Self {
        value.0.get() as u32
    }
}

impl TryFrom<u32> for SpellId {
    type Error = InvalidCardId;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Ok(Some(value)) = <u16 as TryFrom<u32>>::try_from(value).map(NonZeroU16::new) {
            if value.get() <= (SPELL_CARDS.len() as u16) {
                return Ok(Self(value));
            }
        }

        Err(InvalidCardId::InvalidCard(
            GameId::IN,
            value,
            SPELL_CARDS.len() as u32,
        ))
    }
}

impl Display for SpellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl GameValue for SpellId {
    type RawValue = u32;
    type ConversionError = InvalidCardId;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u32 {
        (*self).into()
    }

    fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidCardId::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.card_info().name
    }
}

fn invalid_difficulty(value: u64) -> InvalidDifficultyId {
    InvalidDifficultyId::InvalidDifficulty(GameId::IN, value as u16, 5)
}

#[derive(Debug, NumericEnum, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[error_type = "InvalidDifficultyId"]
#[convert_error = "invalid_difficulty"]
pub enum Difficulty {
    Easy = 0,
    Normal = 1,
    Hard = 2,
    Lunatic = 3,
    Extra = 4,
    LastWord = 5,
}

impl GameValue for Difficulty {
    type RawValue = u16;
    type ConversionError = InvalidDifficultyId;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidDifficultyId> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidDifficultyId::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_stage(value: u64) -> InvalidStageId {
    InvalidStageId::InvalidStage(GameId::IN, value as u16, 8)
}

#[derive(Debug, NumericEnum, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[error_type = "InvalidStageId"]
#[convert_error = "invalid_stage"]
pub enum Stage {
    #[name = "Stage 1"]
    One = 0,
    #[name = "Stage 2"]
    Two = 1,
    #[name = "Stage 3"]
    Three = 2,
    #[name = "Stage 4 Uncanny"]
    FourA = 3,
    #[name = "Stage 4 Powerful"]
    FourB = 4,
    #[name = "Stage 5"]
    Five = 5,
    #[name = "Final A"]
    FinalA = 6,
    #[name = "Final B"]
    FinalB = 7,
    #[name = "Extra Stage"]
    Extra = 8,
    #[name = "Last Word"]
    LastWord = 9,
}

impl GameValue for Stage {
    type RawValue = u16;
    type ConversionError = InvalidStageId;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidStageId> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidStageId::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou8;

impl Touhou8 {
    pub const SHOT_TYPES: &[WrappedShot<Touhou8>; 12] = &[
        WrappedShot::new(ShotType::BarrierTeam),
        WrappedShot::new(ShotType::MagicTeam),
        WrappedShot::new(ShotType::ScarletTeam),
        WrappedShot::new(ShotType::GhostTeam),
        WrappedShot::new(ShotType::Reimu),
        WrappedShot::new(ShotType::Yukari),
        WrappedShot::new(ShotType::Marisa),
        WrappedShot::new(ShotType::Alice),
        WrappedShot::new(ShotType::Sakuya),
        WrappedShot::new(ShotType::Remilia),
        WrappedShot::new(ShotType::Youmu),
        WrappedShot::new(ShotType::Yuyuko),
    ];
}

#[cfg(feature = "find-process")]
impl Touhou8 {
    pub fn find_process(system: &System) -> Option<&Process> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find(|&process| {
                process
                    .exe()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("th08"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}

impl Game for Touhou8 {
    type SpellID = SpellId;
    type ShotTypeID = ShotType;
    type DifficultyID = Difficulty;
    type StageID = Stage;
    type SpellCardRecord = score::SpellCardData;
    type PracticeRecord = score::PracticeScore;
    type ScoreFile = score::ScoreFile;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn card_info(id: SpellId) -> &'static SpellCardInfo<Self> {
        id.card_info()
    }
}
