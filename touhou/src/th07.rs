use std::fmt::Display;
use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};
#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, System, SystemExt};

use crate::types::{
    Character, Game, GameId, GameValue, InvalidCardId, InvalidDifficultyId, InvalidShotType,
    InvalidStageId, ShotType as WrappedShot, SpellCardInfo,
};

#[cfg(feature = "memory")]
pub mod memory;

// pub mod replay;

pub mod spellcards;

#[cfg(feature = "score-file")]
pub mod score;

#[cfg(feature = "memory")]
pub use memory::{GameMemory, StageLocation};
#[cfg(feature = "score-file")]
pub use score::{PracticeData, ScoreFile, SpellCardData};
use spellcards::SPELL_CARDS;
use touhou_macros::NumericEnum;

fn invalid_shot_type(value: u64) -> InvalidShotType {
    InvalidShotType::InvalidShotId(value as u16, 0, 5)
}

#[derive(Debug, NumericEnum)]
#[error_type = "InvalidShotType"]
#[convert_error = "invalid_shot_type"]
pub enum ShotType {
    #[name = "Reimu A"]
    ReimuA = 0,
    #[name = "Reimu B"]
    ReimuB = 1,
    #[name = "Marisa A"]
    MarisaA = 2,
    #[name = "Marisa B"]
    MarisaB = 3,
    #[name = "Sakuya A"]
    SakuyaA = 4,
    #[name = "Sakuya B"]
    SakuyaB = 5,
}

impl ShotType {
    pub fn character(self) -> Character {
        match self {
            Self::ReimuA | Self::ReimuB => Character::Reimu,
            Self::MarisaA | Self::MarisaB => Character::Marisa,
            Self::SakuyaA | Self::SakuyaB => Character::Sakuya,
        }
    }

    pub fn is_type_a(self) -> bool {
        matches!(self, Self::ReimuA | Self::MarisaA | Self::SakuyaA)
    }

    pub fn is_type_b(self) -> bool {
        !self.is_type_a()
    }
}

impl GameValue for ShotType {
    type RawValue = u16;
    type ConversionError = InvalidShotType;

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidShotType::UnexpectedGameId(game, GameId::PCB))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellId(NonZeroU16);

impl SpellId {
    pub fn card_info(&self) -> &'static SpellCardInfo<Touhou7> {
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
            GameId::PCB,
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
        GameId::PCB
    }

    fn raw_id(&self) -> u32 {
        (*self).into()
    }

    fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidCardId::UnexpectedGameId(game, GameId::PCB))
        }
    }

    fn name(&self) -> &'static str {
        self.card_info().name
    }
}

fn invalid_difficulty(value: u64) -> InvalidDifficultyId {
    InvalidDifficultyId::InvalidDifficulty(GameId::PCB, value as u16, 5)
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
    Phantasm = 5,
}

impl GameValue for Difficulty {
    type RawValue = u16;
    type ConversionError = InvalidDifficultyId;

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidDifficultyId> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidDifficultyId::UnexpectedGameId(game, GameId::PCB))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_stage(value: u64) -> InvalidStageId {
    InvalidStageId::InvalidStage(GameId::PCB, value as u16, 7)
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
    #[name = "Stage 4"]
    Four = 3,
    #[name = "Stage 5"]
    Five = 4,
    #[name = "Stage 6"]
    Six = 5,
    #[name = "Extra Stage"]
    Extra = 6,
    #[name = "Phantasm Stage"]
    Phantasm = 7,
}

impl GameValue for Stage {
    type RawValue = u16;
    type ConversionError = InvalidStageId;

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidStageId> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidStageId::UnexpectedGameId(game, GameId::PCB))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou7;

impl Touhou7 {
    pub const SHOT_TYPES: &[WrappedShot<Touhou7>; 6] = &[
        WrappedShot::new(ShotType::ReimuA),
        WrappedShot::new(ShotType::ReimuB),
        WrappedShot::new(ShotType::MarisaA),
        WrappedShot::new(ShotType::MarisaB),
        WrappedShot::new(ShotType::SakuyaA),
        WrappedShot::new(ShotType::SakuyaB),
    ];

    #[cfg(feature = "score-file")]
    pub fn load_score_file<R: std::io::Read>(src: R) -> Result<score::ScoreFile, std::io::Error> {
        ScoreFile::new(src)
    }
}

#[cfg(feature = "find-process")]
impl Touhou7 {
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
                    .map(|s| s.starts_with("th07"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}

impl Game for Touhou7 {
    type SpellID = SpellId;
    type ShotTypeID = ShotType;
    type DifficultyID = Difficulty;
    type StageID = Stage;

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn card_info(id: SpellId) -> &'static SpellCardInfo<Self> {
        id.card_info()
    }
}
