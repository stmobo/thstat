#![feature(doc_auto_cfg)]

#[cfg(feature = "score-file")]
pub mod score;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "tracking")]
pub mod tracking;

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
pub use types::{
    AllIterable, Difficulty, Game, GameValue, PowerValue, ShotPower, ShotType, SpellCard, Stage,
};
