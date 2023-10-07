//! Dynamically typed wrappers around game-specific data.
//!
//! This module contains types that can store game-specific data from any game without needing to specify a concrete game as part of the type.
//!
//! These types work by storing values as raw IDs (see [`GameValue::raw_id`]) along with the [`GameId`] of their associated game,
//! and performing runtime checks when converting to and from the concrete game-specific types.
//!
//! Each of these types implements [`GameValue`] much like the corresponding game-specific types, so you can use [`GameValue::name`] to retrieve a name
//! for any value without having to downcast it first.

use super::difficulty::*;
use super::shot_type::*;
use super::spell_card::*;
use super::stage::*;
use super::{Game, GameId, GameValue};

macro_rules! define_any_wrapper {
    ($wrapper_name:ident, $value_assoc_ty:ident, $specific_wrapper:ident, $specific_ty:ident, $raw_ty:ident, $err_ty:ident) => {
        impl $wrapper_name {
            /// Wrap a game-specific value inside this type.
            pub fn new<G: Game>(id: G::$value_assoc_ty) -> Self {
                Self {
                    game: G::GAME_ID,
                    id: id.raw_id(),
                }
            }

            /// Get the stored game ID within this value.
            pub const fn game_id(&self) -> GameId {
                self.game
            }

            /// Get the raw data ID stored within this value.
            pub const fn id(&self) -> $raw_ty {
                self.id
            }

            /// Get a human-friendly name associated with this value.
            ///
            /// This is equivalent to downcasting to a game-specific type and calling its `name` method.
            ///
            /// # Panics
            ///
            /// Panics if the stored value is for a game for which support has not been enabled in crate features
            /// (for example, a value from PCB without the `th07` feature).
            pub fn name(&self) -> &'static str {
                use super::VisitGame;

                struct Visitor($raw_ty);

                impl VisitGame for Visitor {
                    type Output = &'static str;

                    #[cfg(feature = "th07")]
                    fn visit_th07(self) -> Self::Output {
                        use crate::th07::$specific_ty;
                        $specific_ty::try_from(self.0).unwrap().name()
                    }

                    #[cfg(feature = "th08")]
                    fn visit_th08(self) -> Self::Output {
                        use crate::th08::$specific_ty;
                        $specific_ty::try_from(self.0).unwrap().name()
                    }

                    #[cfg(feature = "th10")]
                    fn visit_th10(self) -> Self::Output {
                        use crate::th10::$specific_ty;
                        $specific_ty::try_from(self.0).unwrap().name()
                    }

                    #[cfg(feature = "th15")]
                    fn visit_th15(self) -> Self::Output {
                        use crate::th15::$specific_ty;
                        $specific_ty::try_from(self.0).unwrap().name()
                    }
                }

                Visitor(self.id).accept_id(self.game)
            }

            pub(crate) fn downcast_id<G: Game>(
                self,
            ) -> Result<G::$value_assoc_ty, crate::types::errors::$err_ty> {
                <G::$value_assoc_ty as GameValue>::from_raw(self.id, self.game)
            }

            pub fn downcast<G: Game>(
                self,
            ) -> Result<$specific_wrapper<G>, crate::types::errors::$err_ty> {
                self.downcast_id::<G>().map($specific_wrapper::new)
            }
        }

        impl<G: Game> TryFrom<$wrapper_name> for $specific_wrapper<G> {
            type Error = crate::types::errors::$err_ty;

            fn try_from(
                value: $wrapper_name,
            ) -> Result<$specific_wrapper<G>, crate::types::errors::$err_ty> {
                value.downcast()
            }
        }

        impl GameValue for $wrapper_name {
            type ConversionError = crate::types::errors::$err_ty;
            type RawValue = $raw_ty;

            fn game_id(&self) -> GameId {
                self.game
            }

            fn raw_id(&self) -> $raw_ty {
                self.id
            }

            fn from_raw(id: $raw_ty, game: GameId) -> Result<Self, Self::ConversionError> {
                use super::VisitGame;

                struct Visitor($raw_ty);

                impl VisitGame for Visitor {
                    type Output = Option<crate::types::errors::$err_ty>;

                    #[cfg(feature = "th07")]
                    fn visit_th07(self) -> Self::Output {
                        use crate::th07::$specific_ty;
                        $specific_ty::try_from(self.0).err()
                    }

                    #[cfg(not(feature = "th07"))]
                    fn visit_th07(self) -> Self::Output {
                        Some(crate::types::errors::$err_ty::UnsupportedGameId(
                            GameId::PCB,
                        ))
                    }

                    #[cfg(feature = "th08")]
                    fn visit_th08(self) -> Self::Output {
                        use crate::th08::$specific_ty;
                        $specific_ty::try_from(self.0).err()
                    }

                    #[cfg(not(feature = "th08"))]
                    fn visit_th08(self) -> Self::Output {
                        Some(crate::types::errors::$err_ty::UnsupportedGameId(GameId::IN))
                    }

                    #[cfg(feature = "th10")]
                    fn visit_th10(self) -> Self::Output {
                        use crate::th10::$specific_ty;
                        $specific_ty::try_from(self.0).err()
                    }

                    #[cfg(not(feature = "th10"))]
                    fn visit_th10(self) -> Self::Output {
                        Some(crate::types::errors::$err_ty::UnsupportedGameId(
                            GameId::MoF,
                        ))
                    }

                    #[cfg(feature = "th15")]
                    fn visit_th15(self) -> Self::Output {
                        use crate::th15::$specific_ty;
                        $specific_ty::try_from(self.0).err()
                    }

                    #[cfg(not(feature = "th15"))]
                    fn visit_th15(self) -> Self::Output {
                        Some(crate::types::errors::$err_ty::UnsupportedGameId(
                            GameId::LoLK,
                        ))
                    }
                }

                if let Some(err) = Visitor(id).accept_id(game) {
                    Err(err)
                } else {
                    Ok(Self { game, id })
                }
            }

            fn name(&self) -> &'static str {
                self.name()
            }
        }

        impl std::fmt::Display for $wrapper_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.pad(self.name())
            }
        }
    };
}

/// A dynamically-typed representation of a stage in a Touhou game.
///
/// See the [module documentation](`self`) for more details.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AnyStage {
    game: super::GameId,
    id: u16,
}

define_any_wrapper!(AnyStage, StageID, Stage, Stage, u16, InvalidStageId);

/// A dynamically-typed representation of a spell card in a Touhou game.
///
/// See the [module documentation](`self`) for more details.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AnySpellCard {
    game: super::GameId,
    id: u32,
}

define_any_wrapper!(
    AnySpellCard,
    SpellID,
    SpellCard,
    SpellId,
    u32,
    InvalidCardId
);

/// A dynamically-typed representation of a selectable difficulty in a Touhou game.
///
/// See the [module documentation](`self`) for more details.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AnyDifficulty {
    game: super::GameId,
    id: u16,
}

define_any_wrapper!(
    AnyDifficulty,
    DifficultyID,
    Difficulty,
    Difficulty,
    u16,
    InvalidDifficultyId
);

/// A dynamically-typed representation of a selectable shot type in a Touhou game.
///
/// See the [module documentation](`self`) for more details.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AnyShotType {
    game: super::GameId,
    id: u16,
}

define_any_wrapper!(
    AnyShotType,
    ShotTypeID,
    ShotType,
    ShotType,
    u16,
    InvalidShotType
);
