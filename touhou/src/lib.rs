#![feature(doc_auto_cfg)]

#[cfg(feature = "score-file")]
pub mod crypt;

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
#[cfg(feature = "th15")]
pub mod th15;

#[cfg(feature = "memory")]
#[doc(inline)]
pub use memory::{HasLocations, Location};
#[cfg(feature = "th07")]
#[doc(inline)]
pub use th07::Touhou7;
#[cfg(feature = "th08")]
#[doc(inline)]
pub use th08::Touhou8;
#[cfg(feature = "th10")]
#[doc(inline)]
pub use th10::Touhou10;
#[cfg(feature = "th15")]
#[doc(inline)]
pub use th15::Touhou15;

pub mod types;

#[doc(inline)]
pub use types::{Difficulty, Game, ShotPower, ShotType, SpellCard, Stage};
#[cfg(feature = "score-file")]
#[doc(inline)]
pub use types::{PracticeRecord, ScoreFile, SpellCardRecord, SpellPracticeRecord};
