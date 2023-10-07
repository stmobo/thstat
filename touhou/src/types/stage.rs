//! Types for working with stages.

use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use super::{impl_wrapper_traits, Game, GameValue};

/// Represents a stage from one of the Touhou games.
///
/// This is a convenience wrapper around the game-specific stage enumerations defined elsewhere in this crate.
/// To access the inner type, use [`Self::unwrap`].
#[repr(transparent)]
pub struct Stage<G: Game>(G::StageID);

impl<G: Game> Stage<G> {
    /// Wraps a game-specific stage enumeration inside of a `Stage`.
    pub const fn new(id: G::StageID) -> Self {
        Self(id)
    }

    /// Gets the inner enumeration type from this wrapper.
    pub const fn unwrap(self) -> G::StageID {
        self.0
    }
}

impl<G: Game> AsRef<G::StageID> for Stage<G> {
    fn as_ref(&self) -> &G::StageID {
        &self.0
    }
}

impl<G: Game> Deref for Stage<G> {
    type Target = G::StageID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl_wrapper_traits!(Stage, u16, G::StageID, IterAll);

impl<G: Game> Debug for Stage<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stage<{}>({:?})",
            self.0.game_id().abbreviation(),
            self.0
        )
    }
}

impl<G: Game> Display for Stage<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.0.name())
    }
}

#[derive(Debug)]
pub enum StageProgress<G: Game> {
    NotStarted,
    LostAt(Stage<G>),
    StageCleared(Stage<G>),
    AllClear,
}

impl<G: Game> Display for StageProgress<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => f.pad("Not Started"),
            Self::LostAt(s) => <Stage<G> as Display>::fmt(s, f),
            Self::AllClear => f.pad("All Clear"),
            Self::StageCleared(s) => write!(f, "{} Cleared", s),
        }
    }
}

impl<G: Game> Clone for StageProgress<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Game> Copy for StageProgress<G> {}

impl<G: Game> PartialEq for StageProgress<G> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotStarted, Self::NotStarted) | (Self::AllClear, Self::AllClear) => true,
            (Self::LostAt(a), Self::LostAt(b)) | (Self::StageCleared(a), Self::StageCleared(b)) => {
                a.eq(b)
            }
            _ => false,
        }
    }
}

impl<G: Game> Eq for StageProgress<G> {}

impl<G: Game> Ord for StageProgress<G> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::NotStarted => {
                if let Self::NotStarted = other {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            }
            Self::LostAt(a) => match other {
                Self::NotStarted => Ordering::Greater,
                Self::LostAt(b) => a.cmp(b),
                Self::StageCleared(_) | Self::AllClear => Ordering::Less,
            },
            Self::StageCleared(a) => match other {
                Self::NotStarted | Self::LostAt(_) => Ordering::Greater,
                Self::StageCleared(b) => a.cmp(b),
                Self::AllClear => Ordering::Less,
            },
            Self::AllClear => {
                if let Self::AllClear = other {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            }
        }
    }
}

impl<G: Game> PartialOrd for StageProgress<G> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Game> Hash for StageProgress<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::NotStarted => 0u8.hash(state),
            Self::LostAt(stage) => {
                1u8.hash(state);
                stage.hash(state);
            }
            Self::StageCleared(stage) => {
                2u8.hash(state);
                stage.hash(state);
            }
            Self::AllClear => 3u8.hash(state),
        }
    }
}
