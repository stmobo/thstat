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
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidCardId {
    #[error("Invalid card ID {1} for {0} (valid values are 1..={2})")]
    InvalidCard(GameId, u32, u32),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidStageId {
    #[error("Invalid stage ID {1} for {0} (valid values are 0..{2})")]
    InvalidStage(GameId, u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
}

#[derive(Debug, Clone, Copy, Error)]
pub enum InvalidDifficultyId {
    #[error("Invalid difficulty ID {1} for {0} (valid values are 0..{2})")]
    InvalidDifficulty(GameId, u16, u16),
    #[error("Invalid game ID {0}")]
    InvalidGameId(u8),
    #[error("Incorrect game ID {0} (expected {1})")]
    UnexpectedGameId(GameId, GameId),
}
