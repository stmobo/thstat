use serde::{Deserialize, Serialize};
use touhou_macros::NumericEnum;

#[cfg(feature = "memory")]
pub mod memory;

mod spellcards;

pub use spellcards::SpellId;

use crate::types::{
    Game, GameId, GameValue, InvalidDifficultyId, InvalidShotType, InvalidStageId,
    ShotType as WrappedShot, SpellCardInfo,
};

fn invalid_shot_type(value: u64) -> InvalidShotType {
    InvalidShotType::InvalidShotId(value as u16, 0, 5)
}

#[derive(Debug, NumericEnum)]
#[error_type = "InvalidShotType"]
#[convert_error = "invalid_shot_type"]
pub enum ShotType {
    ReimuA = 0,
    ReimuB = 1,
    ReimuC = 2,
    MarisaA = 3,
    MarisaB = 4,
    MarisaC = 5,
}

impl GameValue for ShotType {
    type RawValue = u16;
    type ConversionError = InvalidShotType;

    fn game_id(&self) -> GameId {
        GameId::MoF
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
        if game == GameId::MoF {
            id.try_into()
        } else {
            Err(InvalidShotType::UnexpectedGameId(game, GameId::MoF))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_difficulty(value: u64) -> InvalidDifficultyId {
    InvalidDifficultyId::InvalidDifficulty(GameId::MoF, value as u16, 4)
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
}

impl GameValue for Difficulty {
    type RawValue = u16;
    type ConversionError = InvalidDifficultyId;

    fn game_id(&self) -> GameId {
        GameId::MoF
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidDifficultyId> {
        if game == GameId::MoF {
            id.try_into()
        } else {
            Err(InvalidDifficultyId::UnexpectedGameId(game, GameId::MoF))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_stage(value: u64) -> InvalidStageId {
    InvalidStageId::InvalidStage(GameId::MoF, value as u16, 6)
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
}

impl GameValue for Stage {
    type RawValue = u16;
    type ConversionError = InvalidStageId;

    fn game_id(&self) -> GameId {
        GameId::MoF
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidStageId> {
        if game == GameId::MoF {
            id.try_into()
        } else {
            Err(InvalidStageId::UnexpectedGameId(game, GameId::MoF))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou10;

impl Touhou10 {
    pub const SHOT_TYPES: &[WrappedShot<Touhou10>; 6] = &[
        WrappedShot::new(ShotType::ReimuA),
        WrappedShot::new(ShotType::ReimuB),
        WrappedShot::new(ShotType::ReimuC),
        WrappedShot::new(ShotType::MarisaA),
        WrappedShot::new(ShotType::MarisaB),
        WrappedShot::new(ShotType::MarisaC),
    ];
}

impl Game for Touhou10 {
    type SpellID = SpellId;
    type ShotTypeID = ShotType;
    type DifficultyID = Difficulty;
    type StageID = Stage;

    fn game_id(&self) -> GameId {
        GameId::MoF
    }

    fn card_info(id: SpellId) -> &'static SpellCardInfo<Self> {
        id.card_info()
    }
}
