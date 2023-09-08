use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;



use super::{impl_wrapper_traits, Game, GameId, GameValue};

#[repr(transparent)]
pub struct ShotType<G: Game>(G::ShotTypeID);

impl<G: Game> ShotType<G> {
    pub const fn new(id: G::ShotTypeID) -> Self {
        Self(id)
    }

    pub fn unwrap(self) -> G::ShotTypeID {
        self.0
    }
}

impl<G: Game> AsRef<G::ShotTypeID> for ShotType<G> {
    fn as_ref(&self) -> &G::ShotTypeID {
        &self.0
    }
}

impl<G: Game> Deref for ShotType<G> {
    type Target = G::ShotTypeID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl_wrapper_traits!(ShotType, u16, G::ShotTypeID);

impl<G: Game> Debug for ShotType<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ShotType<{}>({:?})",
            self.0.game_id().abbreviation(),
            self.0
        )
    }
}

impl<G: Game> Display for ShotType<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.name())
    }
}
