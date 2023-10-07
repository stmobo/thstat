use std::error::Error;
use std::fmt::Display;

use thiserror::Error;

use super::GameId;

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidShotType {
    #[error("Invalid shot ID {1} for {0} (valid values are 0..{2})")]
    InvalidShotId(GameId, u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
    #[error("Support not compiled for {0}")]
    UnsupportedGameId(GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidCardId {
    #[error("Invalid card ID {1} for {0} (valid values are 1..={2})")]
    InvalidCard(GameId, u32, u32),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
    #[error("Support not compiled for {0}")]
    UnsupportedGameId(GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidStageId {
    #[error("Invalid stage ID {1} for {0} (valid values are 0..{2})")]
    InvalidStage(GameId, u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
    #[error("Support not compiled for {0}")]
    UnsupportedGameId(GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidDifficultyId {
    #[error("Invalid difficulty ID {1} for {0} (valid values are 0..{2})")]
    InvalidDifficulty(GameId, u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
    #[error("Support not compiled for {0}")]
    UnsupportedGameId(GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidShotPower {
    #[error("Invalid shot power {0} (valid values are 0..{1})")]
    InvalidPower(u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
    #[error("Support not compiled for {0}")]
    UnsupportedGameId(GameId),
}

#[derive(Debug, Copy, Clone)]
pub struct InvalidGameId(u8);

impl InvalidGameId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl Display for InvalidGameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid game ID {}", self.0)
    }
}

impl Error for InvalidGameId {}
