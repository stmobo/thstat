pub mod crypt;
pub mod decompress;
pub mod types;

pub mod th07;
pub mod th08;

pub use th07::Touhou7;
pub use th08::Touhou8;
pub use types::{
    Difficulty, Game, PracticeRecord, ScoreFile, ShotType, SpellCard, SpellCardRecord,
    SpellPracticeRecord, Stage,
};
