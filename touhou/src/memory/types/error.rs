use std::error::Error;
use std::fmt::Display;
use std::io::Error as IOError;
use std::ops::RangeInclusive;

use crate::types::errors::{
    InvalidCardId, InvalidDifficultyId, InvalidGameValue, InvalidShotPower, InvalidShotType,
    InvalidStageId,
};
use crate::types::{Game, GameId};

#[derive(Debug, Clone)]
pub enum InvalidLocationData<G: Game> {
    IncorrectGame(GameId),
    InvalidIndex {
        stage: &'static str,
        index: u64,
        valid: RangeInclusive<u64>,
    },
    MissingSpell {
        stage: &'static str,
        loc_name: &'static str,
        valid: RangeInclusive<u32>,
    },
    InvalidSpell {
        stage: &'static str,
        loc_name: &'static str,
        valid: RangeInclusive<u32>,
    },
    InvalidStage(InvalidStageId<G>),
    NoStageData {
        stage: &'static str,
    },
}

impl<G: Game> Error for InvalidLocationData<G> {}

impl<G: Game> Display for InvalidLocationData<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoStageData { stage } => write!(
                f,
                "No stage data defined for {} stage {}",
                G::GAME_ID.abbreviation(),
                stage
            ),
            Self::IncorrectGame(actual) => write!(
                f,
                "Incorrect game ID {} found (expected {})",
                actual.abbreviation(),
                G::GAME_ID.abbreviation()
            ),
            Self::InvalidIndex {
                stage,
                index,
                valid,
            } => write!(
                f,
                "Invalid location index {} for {} stage {} (valid indices are {:?})",
                index,
                G::GAME_ID.abbreviation(),
                stage,
                valid
            ),
            Self::MissingSpell {
                stage,
                loc_name,
                valid,
            } => {
                write!(
                    f,
                    "No spell ID provided for {} stage {} {} (valid spell IDs are {:?})",
                    G::GAME_ID.abbreviation(),
                    stage,
                    loc_name,
                    valid
                )
            }
            Self::InvalidSpell {
                stage,
                loc_name,
                valid,
            } => write!(
                f,
                "Invalid spell ID provided for {} stage {} {} (valid spell IDs are {:?})",
                G::GAME_ID.abbreviation(),
                stage,
                loc_name,
                valid
            ),
            Self::InvalidStage(err) => err.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum MemoryReadError<G: Game> {
    IO(IOError),
    InvalidStage(InvalidStageId<G>),
    InvalidShotType(InvalidShotType<G>),
    InvalidPowerValue(InvalidShotPower<G>),
    InvalidDifficulty(InvalidDifficultyId<G>),
    InvalidSpellCard(InvalidCardId<G>),
    InvalidFloat(InvalidGameValue<f32>),
    InvalidOther(InvalidGameValue<i64>),
    Other(String),
}

impl<G: Game> MemoryReadError<G> {
    pub fn other<T: std::fmt::Display>(message: T) -> Self {
        Self::Other(message.to_string())
    }

    pub const fn float_out_of_range(
        type_name: &'static str,
        value: f32,
        valid_start: f32,
        valid_end: f32,
    ) -> Self {
        Self::InvalidFloat(InvalidGameValue::out_of_range(
            type_name,
            G::GAME_ID,
            value,
            valid_start,
            valid_end,
        ))
    }

    pub fn other_out_of_range<T: Into<i64>>(
        type_name: &'static str,
        value: T,
        valid_start: T,
        valid_end: T,
    ) -> Self {
        Self::InvalidOther(InvalidGameValue::out_of_range(
            type_name,
            G::GAME_ID,
            value.into(),
            valid_start.into(),
            valid_end.into(),
        ))
    }

    pub fn new_other<T: Into<i64>>(err: InvalidGameValue<T>) -> Self {
        Self::InvalidOther(err.into_other())
    }
}

impl<G: Game> From<IOError> for MemoryReadError<G> {
    fn from(value: IOError) -> Self {
        Self::IO(value)
    }
}

impl<G: Game> From<InvalidStageId<G>> for MemoryReadError<G> {
    fn from(value: InvalidStageId<G>) -> Self {
        Self::InvalidStage(value)
    }
}

impl<G: Game> From<InvalidShotType<G>> for MemoryReadError<G> {
    fn from(value: InvalidShotType<G>) -> Self {
        Self::InvalidShotType(value)
    }
}

impl<G: Game> From<InvalidShotPower<G>> for MemoryReadError<G> {
    fn from(value: InvalidShotPower<G>) -> Self {
        Self::InvalidPowerValue(value)
    }
}

impl<G: Game> From<InvalidDifficultyId<G>> for MemoryReadError<G> {
    fn from(value: InvalidDifficultyId<G>) -> Self {
        Self::InvalidDifficulty(value)
    }
}

impl<G: Game> From<InvalidCardId<G>> for MemoryReadError<G> {
    fn from(value: InvalidCardId<G>) -> Self {
        Self::InvalidSpellCard(value)
    }
}

impl<G: Game> From<InvalidGameValue<f32>> for MemoryReadError<G> {
    fn from(value: InvalidGameValue<f32>) -> Self {
        Self::InvalidFloat(value)
    }
}

impl<G: Game> Display for MemoryReadError<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("memory read error: ")?;
        match self {
            Self::IO(err) => err.fmt(f),
            Self::InvalidStage(err) => err.fmt(f),
            Self::InvalidShotType(err) => err.fmt(f),
            Self::InvalidPowerValue(err) => err.fmt(f),
            Self::InvalidDifficulty(err) => err.fmt(f),
            Self::InvalidSpellCard(err) => err.fmt(f),
            Self::InvalidFloat(err) => err.fmt(f),
            Self::InvalidOther(err) => err.fmt(f),
            Self::Other(msg) => msg.fmt(f),
        }
    }
}

impl<G: Game> Error for MemoryReadError<G> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IO(err) => Some(err),
            Self::InvalidStage(err) => Some(err),
            Self::InvalidShotType(err) => Some(err),
            Self::InvalidPowerValue(err) => Some(err),
            Self::InvalidDifficulty(err) => Some(err),
            Self::InvalidSpellCard(err) => Some(err),
            Self::InvalidFloat(err) => Some(err),
            Self::InvalidOther(err) => Some(err),
            Self::Other(_) => None,
        }
    }
}
