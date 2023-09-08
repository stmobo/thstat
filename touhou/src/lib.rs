#[cfg(feature = "score-file")]
pub mod crypt;
pub mod types;

#[cfg(feature = "score-file")]
pub mod decompress;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "th07")]
pub mod th07;
#[cfg(feature = "th08")]
pub mod th08;
#[cfg(feature = "th10")]
pub mod th10;

#[cfg(feature = "th07")]
pub use th07::Touhou7;
#[cfg(feature = "th08")]
pub use th08::Touhou8;
#[cfg(feature = "th10")]
pub use th10::Touhou10;
pub use types::{Difficulty, Game, ShotType, SpellCard, Stage};
#[cfg(feature = "score-file")]
pub use types::{PracticeRecord, ScoreFile, SpellCardRecord, SpellPracticeRecord};
