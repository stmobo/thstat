//! Definitions specific to Touhou 10 (*Mountain of Faith*).

use std::borrow::Borrow;
use std::fmt::Display;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use touhou_macros::define_game;

use crate::types::errors::InvalidShotPower;
use crate::types::PowerValue;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "memory")]
pub use memory::*;

mod spellcards;

pub use spellcards::SpellId;

use crate::types::GameId;

define_game! {
    /// The tenth game in the series: *Touhou Fuujinroku ~ Mountain of Faith*.
    Touhou10 {
        type SpellID = SpellId;
        type ShotPower = Other(ShotPower);
        const GAME_ID = MoF;

        /// The selectable shot types in Touhou 10.
        ShotType {
            ReimuA,
            ReimuB,
            ReimuC,
            MarisaA,
            MarisaB,
            MarisaC,
        }

        /// The selectable difficulty levels in Touhou 10.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Difficulty {
            Easy,
            Normal,
            Hard,
            Lunatic,
            Extra
        }

        /// The playable stages in Touhou 10.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Stage {
            One: "Stage 1",
            Two: "Stage 2",
            Three: "Stage 3",
            Four: "Stage 4",
            Five: "Stage 5",
            Six: "Stage 6",
            Extra: "Extra Stage"
        }
    }
}

/// Represents a shot power value from Touhou 10.
///
/// Similarly to later games in the series, shot power in this game is a decimal value; unlike other games, however, shot power in MoF is always a multiple of 0.05.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "u16", into = "u16")]
#[repr(transparent)]
pub struct ShotPower(u16);

impl ShotPower {
    pub const MAX_POWER: Self = Self(100);

    /// Constructs a new instance from a raw power value representing increments of 0.05.
    ///
    /// Valid power values range from 0 to 100 inclusive, corresponding to powers of 0.00 and 5.00 respectively.
    pub const fn new(value: u16) -> Result<Self, InvalidShotPower<Touhou10>> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::out_of_range(value, 0..=100))
        }
    }

    /// Gets the raw power value from this type.
    pub const fn unwrap(self) -> u16 {
        self.0
    }

    /// Gets whether this power value represents maximum power.
    pub const fn is_max(self) -> bool {
        self.0 == 100
    }
}

impl PowerValue<Touhou10> for ShotPower {
    const MAX_POWER: Self = Self(100);

    type RawValue = u16;

    fn new(value: u16) -> Result<Self, InvalidShotPower<Touhou10>> {
        if value <= 100 {
            Ok(Self(value))
        } else {
            Err(InvalidShotPower::out_of_range(value, 0..=100))
        }
    }

    fn unwrap(self) -> u16 {
        self.0
    }

    fn is_max(self) -> bool {
        self.0 == 100
    }
}

impl TryFrom<u16> for ShotPower {
    type Error = InvalidShotPower<Touhou10>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ShotPower> for u16 {
    fn from(value: ShotPower) -> Self {
        value.0
    }
}

impl From<ShotPower> for f64 {
    fn from(value: ShotPower) -> Self {
        (value.0 as Self) * 0.05
    }
}

impl From<ShotPower> for f32 {
    fn from(value: ShotPower) -> Self {
        (value.0 as Self) * 0.05
    }
}

impl AsRef<u16> for ShotPower {
    fn as_ref(&self) -> &u16 {
        &self.0
    }
}

impl Deref for ShotPower {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<u16> for ShotPower {
    fn borrow(&self) -> &u16 {
        &self.0
    }
}

impl PartialEq<u16> for ShotPower {
    fn eq(&self, other: &u16) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<u16> for ShotPower {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(other))
    }
}

impl Display for ShotPower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let whole = self.0 / 20;
        let frac = (self.0 % 20) * 5;
        write!(f, "{}.{:02} / 5.00", whole, frac)
    }
}
