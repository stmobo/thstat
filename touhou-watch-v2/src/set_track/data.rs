use std::hash::Hash;

use serde::{Deserialize, Serialize};
use touhou::memory::Location;
use touhou::{Difficulty, ShotType, Touhou10, Touhou7, Touhou8};

use crate::time::GameTime;
use crate::watcher::TrackedGame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(bound = "G: TrackedGame")]
pub struct SetKey<G: TrackedGame> {
    shot: ShotType<G>,
    difficulty: Difficulty<G>,
    location: Location<G>,
}

impl<G: TrackedGame> SetKey<G> {
    pub(super) fn new(shot: ShotType<G>, difficulty: Difficulty<G>, location: Location<G>) -> Self {
        Self {
            shot,
            difficulty,
            location,
        }
    }

    pub fn shot(&self) -> ShotType<G> {
        self.shot
    }

    pub fn difficulty(&self) -> Difficulty<G> {
        self.difficulty
    }

    pub fn location(&self) -> Location<G> {
        self.location
    }
}

impl<G: TrackedGame> Hash for SetKey<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.shot, self.difficulty, self.location).hash(state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "game", content = "key")]
pub enum MultiSetKey {
    Touhou7(SetKey<Touhou7>),
    Touhou8(SetKey<Touhou8>),
    Touhou10(SetKey<Touhou10>),
}

impl From<SetKey<Touhou7>> for MultiSetKey {
    fn from(value: SetKey<Touhou7>) -> Self {
        Self::Touhou7(value)
    }
}

impl From<SetKey<Touhou8>> for MultiSetKey {
    fn from(value: SetKey<Touhou8>) -> Self {
        Self::Touhou8(value)
    }
}

impl From<SetKey<Touhou10>> for MultiSetKey {
    fn from(value: SetKey<Touhou10>) -> Self {
        Self::Touhou10(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attempt {
    start_time: GameTime,
    end_time: GameTime,
    success: bool,
}

impl Attempt {
    pub const fn new(start_time: GameTime, end_time: GameTime, success: bool) -> Self {
        Self {
            start_time,
            end_time,
            success,
        }
    }
}
