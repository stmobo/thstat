//! Types for working with game difficulty settings.

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use super::{impl_wrapper_traits, Game, GameValue};

/// Represents a selectable difficulty level from one of the Touhou games.
///
/// This is a convenience wrapper around the game-specific difficulty enumerations defined elsewhere in this crate.
/// To access the inner type, use [`Self::unwrap`].
#[repr(transparent)]
pub struct Difficulty<G: Game>(G::DifficultyID);

impl<G: Game> Difficulty<G> {
    /// Wraps a game-specific difficulty enumeration inside of a `Difficulty`.
    pub const fn new(id: G::DifficultyID) -> Self {
        Self(id)
    }

    /// Gets the inner enumeration type from this wrapper.
    pub const fn unwrap(self) -> G::DifficultyID {
        self.0
    }
}

impl<G: Game> AsRef<G::DifficultyID> for Difficulty<G> {
    fn as_ref(&self) -> &G::DifficultyID {
        &self.0
    }
}

impl<G: Game> Deref for Difficulty<G> {
    type Target = G::DifficultyID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl_wrapper_traits!(Difficulty, u16, G::DifficultyID, IterAll);

impl<G: Game> Debug for Difficulty<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Difficulty<{}>({:?})",
            self.0.game_id().abbreviation(),
            self.0
        )
    }
}

impl<G: Game> Display for Difficulty<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.0.name())
    }
}
