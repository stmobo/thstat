//! Types for representing common concepts within the Touhou game series.

use std::error::Error;
use std::fmt::{Debug, Display};
use std::io::{self, ErrorKind, Read};
use std::str;
use std::str::FromStr;

use anyhow::anyhow;

#[doc(hidden)]
pub mod difficulty;
pub mod errors;
#[doc(hidden)]
pub mod game_id;
#[doc(hidden)]
pub mod shot_power;
#[doc(hidden)]
pub mod shot_type;
#[doc(hidden)]
pub mod spell_card;
#[doc(hidden)]
pub mod stage;

#[cfg(feature = "score-file")]
#[doc(hidden)]
pub mod score;

pub mod any;

#[doc(inline)]
pub use difficulty::*;
pub use errors::*;
#[doc(inline)]
pub use game_id::GameId;
#[doc(hidden)]
pub use game_id::InvalidGameId;
pub(crate) use game_id::VisitGame;
#[cfg(feature = "score-file")]
#[doc(inline)]
pub use score::{PracticeRecord, ScoreFile, SpellCardRecord, SpellPracticeRecord};
#[doc(inline)]
pub use shot_power::*;
#[doc(inline)]
pub use shot_type::*;
#[doc(inline)]
pub use spell_card::*;
#[doc(inline)]
pub use stage::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShortDate {
    month: u8,
    day: u8,
}

impl ShortDate {
    pub fn read_from<R: Read>(src: &mut R) -> Result<Self, io::Error> {
        let mut buf = [0; 6];

        src.read_exact(&mut buf)?;
        str::from_utf8(&buf[..5])
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
            .parse()
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }
}

impl Display for ShortDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}/{:02}", self.month, self.day)
    }
}

impl FromStr for ShortDate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((first, last)) = s.split_once('/') {
            let month = first.parse()?;
            let day = last.parse()?;
            Ok(ShortDate { month, day })
        } else {
            Err(anyhow!("could not parse short date {}", s))
        }
    }
}

/// A trait for types representing information specific to individual mainline games.
///
/// This trait is implemented for various types in this crate representing game-specific
/// information such as difficulty levels, spell IDs, playable shot types, and so on.
pub trait GameValue: Debug + Copy + Sync + Send + Unpin + 'static {
    type RawValue;
    type ConversionError: Error;

    /// The ID of the game associated with this value.
    fn game_id(&self) -> GameId;

    /// Returns a raw ID for this value.
    fn raw_id(&self) -> Self::RawValue;

    /// Creates a value from a raw ID and a game ID. Used for deserialization and conversions.
    fn from_raw(id: Self::RawValue, game: GameId) -> Result<Self, Self::ConversionError>;

    /// Gets a human-friendly display name for this value.
    fn name(&self) -> &'static str;
}

/// A trait for types that represent mainline Touhou games.
///
/// This trait ties the different `Touhou` types such as [`crate::th07::Touhou7`] and [`crate::th08::Touhou8`]
/// to the corresponding game-specific types for spell IDs, shot types, stages, and so on.
pub trait Game:
    Sized + Sync + Send + Copy + Eq + Ord + std::hash::Hash + Default + Unpin + 'static
{
    /// The specific [`GameId`] value associated with this game.
    const GAME_ID: GameId;

    type SpellID: GameValue<RawValue = u32, ConversionError = errors::InvalidCardId>;
    type ShotTypeID: GameValue<RawValue = u16, ConversionError = errors::InvalidShotType>;
    type StageID: GameValue<RawValue = u16, ConversionError = errors::InvalidStageId>;
    type DifficultyID: GameValue<RawValue = u16, ConversionError = errors::InvalidDifficultyId>;
    type ShotPower: PowerValue;

    /// Lookup the [`SpellCardInfo`] for a specific spell by ID.
    fn card_info(id: Self::SpellID) -> &'static SpellCardInfo<Self>;
}

macro_rules! impl_wrapper_traits {
    ($t:ident, $val_ty:ty, $wrapped_ty:ty) => {
        impl<G: Game> PartialEq for $t<G> {
            fn eq(&self, other: &Self) -> bool {
                let a: $val_ty = self.0.raw_id();
                let b: $val_ty = other.0.raw_id();
                a == b
            }
        }

        impl<G: Game> Eq for $t<G> {}

        impl<G: Game> PartialOrd for $t<G> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl<G: Game> Ord for $t<G> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let a: $val_ty = self.0.raw_id();
                let b: $val_ty = other.0.raw_id();
                a.cmp(&b)
            }
        }

        impl<G: Game> Hash for $t<G> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                let v: $val_ty = self.0.raw_id();
                v.hash(state)
            }
        }

        impl<G: Game> Clone for $t<G> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<G: Game> Copy for $t<G> {}

        #[derive(serde::Serialize, serde::Deserialize)]
        struct SerializedAs {
            game: $crate::types::GameId,
            id: $val_ty,
        }

        impl<G: Game> serde::Serialize for $t<G> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let serialized = SerializedAs {
                    game: self.0.game_id(),
                    id: self.0.raw_id(),
                };
                <SerializedAs as serde::Serialize>::serialize(&serialized, serializer)
            }
        }

        impl<'de, G: Game> serde::Deserialize<'de> for $t<G> {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let deserialized: SerializedAs =
                    <SerializedAs as serde::Deserialize<'de>>::deserialize(deserializer)?;

                <$wrapped_ty>::from_raw(deserialized.id, deserialized.game)
                    .map(Self)
                    .map_err(<D::Error as serde::de::Error>::custom)
            }
        }
    };
}

pub(super) use impl_wrapper_traits;
