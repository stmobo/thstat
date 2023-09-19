use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

use super::{impl_wrapper_traits, Game, GameValue};

#[repr(transparent)]
pub struct Difficulty<G: Game>(G::DifficultyID);

impl<G: Game> Difficulty<G> {
    pub const fn new(id: G::DifficultyID) -> Self {
        Self(id)
    }

    pub fn unwrap(self) -> G::DifficultyID {
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

impl_wrapper_traits!(Difficulty, u16, G::DifficultyID);

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
        f.write_str(self.0.name())
    }
}
