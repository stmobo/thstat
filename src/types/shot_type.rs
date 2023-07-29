use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;

use thiserror::Error;

use super::{impl_wrapper_traits, Game, IterableEnum};

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid shot type {0} (valid values are {1}..={2})")]
pub struct InvalidShotType(u8, u8, u8);

impl InvalidShotType {
    pub fn new(val: u8, min: u8, max: u8) -> Self {
        Self(val, min, max)
    }
}

#[repr(transparent)]
pub struct ShotType<G: Game>(u8, PhantomData<G::ShotTypeInner>);

impl<G: Game> ShotType<G> {
    pub fn from_inner_type(value: G::ShotTypeInner) -> Self {
        Self(value.into(), PhantomData)
    }

    pub fn as_inner_type(&self) -> G::ShotTypeInner {
        self.0.try_into().unwrap()
    }
}

impl_wrapper_traits!(ShotType, u8);

impl<G: Game> TryFrom<u8> for ShotType<G> {
    type Error = InvalidShotType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        G::ShotTypeInner::try_from(value).map(|_| Self(value, PhantomData))
    }
}

impl<G: Game> Debug for ShotType<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ShotType<{}>({} : {})",
            G::GAME_ID.abbreviation(),
            self.0,
            self.as_inner_type()
        )
    }
}

impl<G: Game> Display for ShotType<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_inner_type().fmt(f)
    }
}
pub struct ShotTypeIter<G: Game>(<G::ShotTypeInner as IterableEnum>::EnumIter);

impl<G: Game> Iterator for ShotTypeIter<G> {
    type Item = ShotType<G>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|i| ShotType(i.into(), PhantomData))
    }
}

impl<G: Game> IterableEnum for ShotType<G> {
    type EnumIter = ShotTypeIter<G>;

    fn iter_all() -> Self::EnumIter {
        ShotTypeIter(<G::ShotTypeInner as IterableEnum>::iter_all())
    }
}
