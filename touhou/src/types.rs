//! Types for representing common concepts within the Touhou game series.

use std::error::Error;
use std::fmt::Debug;
use std::str;

pub mod any;
pub mod difficulty;
pub mod errors;
pub mod game_id;
pub mod shot_power;
pub mod shot_type;
pub mod spell_card;
pub mod stage;

#[doc(inline)]
pub use difficulty::Difficulty;
#[doc(inline)]
pub use game_id::GameId;
pub(crate) use game_id::VisitGame;
#[doc(inline)]
pub use shot_power::{Gen1Power, Gen2Power, PowerValue, ShotPower};
#[doc(inline)]
pub use shot_type::ShotType;
#[doc(inline)]
pub use spell_card::{SpellCard, SpellCardInfo, SpellType};
#[doc(inline)]
pub use stage::{Stage, StageProgress};

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

/// A trait for iterating over all possible values for a type.
///
/// This is implemented by the game-specific value types associated with [`Game`].
pub trait AllIterable: Sized + Copy + Sync + Send + Unpin + 'static {
    type IterAll: Iterator<Item = Self>
        + ExactSizeIterator
        + DoubleEndedIterator
        + std::iter::FusedIterator;

    /// Get an iterator over all possible values for this type.
    fn iter_all() -> Self::IterAll;
}

/// A trait for types that represent mainline Touhou games.
///
/// This trait ties the different `Touhou` types such as [`crate::th07::Touhou7`] and [`crate::th08::Touhou8`]
/// to the corresponding game-specific types for spell IDs, shot types, stages, and so on.
///
/// This crate provides zero-cost convenience wrappers for each of these associated types, with uniform
/// implementations of basic traits such as [`Ord`], [`Eq`], [`Display`](`std::fmt::Display`),
/// and [`Serialize`](`serde::Serialize`) / [`Deserialize`](`serde::Deserialize`).
/// You should generally prefer using those wrappers instead of the associated types here.
pub trait Game:
    Sized
    + Sync
    + Send
    + Copy
    + Eq
    + Ord
    + std::hash::Hash
    + Default
    + Unpin
    + std::fmt::Debug
    + 'static
{
    /// The specific [`GameId`] value associated with this game.
    const GAME_ID: GameId;

    /// The type used to represent this game's spell card IDs.
    ///
    /// [`SpellCard`] wraps this type for more convenient usage.
    /// For more details, see the [trait documentation](`Game`).
    type SpellID: GameValue<RawValue = u32, ConversionError = errors::InvalidCardId<Self>>
        + AllIterable;

    /// The type (typically an enum) used to represent this game's selectable player shot types.
    ///
    /// [`ShotType`] wraps this type for more convenient usage.
    /// For more details, see the [trait documentation](`Game`).
    type ShotTypeID: GameValue<RawValue = u16, ConversionError = errors::InvalidShotType<Self>>
        + AllIterable;

    /// The type (typically an enum) used to represent this game's playable stages.
    ///
    /// [`Stage`] wraps this type for more convenient usage.
    /// For more details, see the [trait documentation](`Game`).
    type StageID: GameValue<RawValue = u16, ConversionError = errors::InvalidStageId<Self>>
        + AllIterable;

    /// The type (typically an enum) used to represent this game's selectable difficulty settings.
    ///
    /// [`Difficulty`] wraps this type for more convenient usage.
    /// For more details, see the [trait documentation](`Game`).
    type DifficultyID: GameValue<RawValue = u16, ConversionError = errors::InvalidDifficultyId<Self>>
        + AllIterable;

    /// The type used to represent the in-game power of a player's shot.
    ///
    /// [`ShotPower`] wraps this type for more convenient usage.
    /// For more details, see the [trait documentation](`Game`).
    type ShotPower: PowerValue<Self>;

    /// Lookup the [`SpellCardInfo`] for a specific spell by ID.
    ///
    /// Note that all `SpellID` types defined by this crate [`Deref`](`std::ops::Deref`)
    /// to [`SpellCardInfo`] instances on their own, so client code shouldn't need to use this.
    fn card_info(id: Self::SpellID) -> &'static SpellCardInfo<Self>;

    /// Gets the abbreviated form of this game's English subtitle.
    ///
    /// For more details, see [`GameId::abbreviation`].
    fn abbreviation() -> &'static str {
        Self::GAME_ID.abbreviation()
    }

    /// Gets a name for this game in the format “Touhou N”, where “N” is the number of this game.
    ///
    /// For more details, see [`GameId::numbered_name`].
    fn numbered_name() -> &'static str {
        Self::GAME_ID.numbered_name()
    }

    /// Gets a romanized form of the Japanese title for this game.
    ///
    /// For more details, see [`GameId::title`].
    fn title() -> &'static str {
        Self::GAME_ID.title()
    }

    /// Gets the English subtitle for this game.
    ///
    /// For more details, see [`GameId::subtitle`].
    fn subtitle() -> &'static str {
        Self::GAME_ID.subtitle()
    }

    /// Gets the full title for this game.
    ///
    /// For more details, see [`GameId::full_title`].
    fn full_title() -> &'static str {
        Self::GAME_ID.full_title()
    }
}

macro_rules! impl_wrapper_traits {
    ($t:ident, $val_ty:ty, $wrapped_ty:ty, $iter_ty:ident) => {
        impl<G1: Game, G2: Game> PartialEq<$t<G2>> for $t<G1> {
            fn eq(&self, other: &$t<G2>) -> bool {
                if G1::GAME_ID == G2::GAME_ID {
                    let a: $val_ty = self.0.raw_id();
                    let b: $val_ty = other.0.raw_id();
                    a == b
                } else {
                    false
                }
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

        #[doc = concat!("An iterator over all possible [`", stringify!($t), "`] values for a given [`Game`].")]
        #[repr(transparent)]
        pub struct $iter_ty<G: Game>(<$wrapped_ty as super::AllIterable>::IterAll);

        impl<G> std::fmt::Debug for $iter_ty<G>
        where
            G: Game,
            <$wrapped_ty as super::AllIterable>::IterAll: std::fmt::Debug
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl<G: Game> Iterator for $iter_ty<G> {
            type Item = $t<G>;

            #[inline]
            fn next(&mut self) -> Option<$t<G>> {
                self.0.next().map($t::new)
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }
        }

        impl<G: Game> DoubleEndedIterator for $iter_ty<G> {
            #[inline]
            fn next_back(&mut self) -> Option<$t<G>> {
                self.0.next_back().map($t::new)
            }
        }

        impl<G: Game> ExactSizeIterator for $iter_ty<G> {
            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }
        }

        impl<G: Game> std::iter::FusedIterator for $iter_ty<G> {}

        impl<G: Game> crate::types::AllIterable for $t<G> {
            type IterAll = $iter_ty<G>;

            fn iter_all() -> $iter_ty<G> {
                $iter_ty(<$wrapped_ty as super::AllIterable>::iter_all())
            }
        }
    };
}

pub(super) use impl_wrapper_traits;
