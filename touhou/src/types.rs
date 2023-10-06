//! Types for representing common concepts within the Touhou game series.

use std::error::Error;
use std::fmt::Debug;
use std::str;

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

pub mod any;

#[doc(inline)]
pub use difficulty::*;
pub use errors::*;
#[doc(inline)]
pub use game_id::GameId;
#[doc(hidden)]
pub use game_id::InvalidGameId;
pub(crate) use game_id::VisitGame;
#[doc(inline)]
pub use shot_power::*;
#[doc(inline)]
pub use shot_type::*;
#[doc(inline)]
pub use spell_card::*;
#[doc(inline)]
pub use stage::*;

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
