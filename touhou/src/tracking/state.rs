//! Types for the different kinds of state that can be tracked using [`TrackerState`](super::TrackerState).

use std::marker::PhantomData;
use std::time::Duration;

use super::EventTime;
use crate::memory::traits::{
    BombCount, BombStock, ContinueCount, LifeStock, MissCount, PauseState, PlayerData,
};
use crate::types::{Game, ShotPower};
use crate::{HasLocations, Location};

mod private {
    pub trait TrackLives {}

    pub trait TrackBombs {}

    pub trait TrackPause {
        fn is_paused(&self) -> bool;
    }

    pub trait TrackContinues {}
}

pub(super) use private::{TrackBombs, TrackContinues, TrackLives, TrackPause};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NotTracked(PhantomData<()>);

impl TrackLives for NotTracked {}
impl TrackBombs for NotTracked {}
impl TrackPause for NotTracked {
    fn is_paused(&self) -> bool {
        false
    }
}
impl TrackContinues for NotTracked {}

impl NotTracked {
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentLives(u8);

impl TrackLives for CurrentLives {}

impl CurrentLives {
    pub(super) fn new<G: Game, S: LifeStock<G>>(state: &S) -> Self {
        Self(state.lives())
    }

    pub(super) fn update<G: Game, S: LifeStock<G>>(&mut self, state: &S) -> bool {
        let new = state.lives();
        let old = std::mem::replace(&mut self.0, new);
        new < old
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TotalMisses(u8);

impl TrackLives for TotalMisses {}

impl TotalMisses {
    pub(super) fn new<G: Game, S: MissCount<G>>(state: &S) -> Self {
        Self(state.total_misses())
    }

    pub(super) fn update<G: Game, S: MissCount<G>>(&mut self, state: &S) -> bool {
        let new = state.total_misses();
        let old = std::mem::replace(&mut self.0, new);
        new > old
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentBombs(u8);

impl TrackBombs for CurrentBombs {}

impl CurrentBombs {
    pub(super) fn new<G: Game, S: BombStock<G>>(state: &S) -> Self {
        Self(state.bombs())
    }

    pub(super) fn update<G: Game, S: BombStock<G>>(&mut self, state: &S) -> bool {
        let new = state.bombs();
        let old = std::mem::replace(&mut self.0, new);
        new < old
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TotalBombsUsed(u8);

impl TrackBombs for TotalBombsUsed {}

impl TotalBombsUsed {
    pub(super) fn new<G: Game, S: BombCount<G>>(state: &S) -> Self {
        Self(state.total_bombs())
    }

    pub(super) fn update<G: Game, S: BombCount<G>>(&mut self, state: &S) -> bool {
        let new = state.total_bombs();
        let old = std::mem::replace(&mut self.0, new);
        new > old
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentPower<G: Game>(ShotPower<G>);

impl<G: Game> TrackBombs for CurrentPower<G> {}

impl<G: Game> CurrentPower<G> {
    pub(super) fn new<S: PlayerData<G>>(state: &S) -> Self {
        Self(state.power())
    }

    pub(super) fn update<S: PlayerData<G>>(&mut self, state: &S, has_missed: bool) -> bool {
        let new = state.power();
        let old = std::mem::replace(&mut self.0, new);
        (new < old) && !has_missed
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ContinuesUsed(u8);

impl TrackContinues for ContinuesUsed {}

impl ContinuesUsed {
    pub(super) fn new<G: Game, S: ContinueCount<G>>(state: &S) -> Self {
        Self(state.continues_used())
    }

    pub(super) fn update<G: Game, S: ContinueCount<G>>(&mut self, state: &S) -> bool {
        let new = state.continues_used();
        let old = std::mem::replace(&mut self.0, new);
        new > old
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentPause(bool);

impl TrackPause for CurrentPause {
    fn is_paused(&self) -> bool {
        self.0
    }
}

impl CurrentPause {
    pub(super) fn new<S: PauseState>(state: &S) -> Self {
        Self(state.paused())
    }
}

#[derive(Debug, Clone)]
pub struct LocationResolveFilter<G: HasLocations> {
    min_time: Duration,
    last_location: (EventTime, Location<G>),
    actual_location: Option<(EventTime, Location<G>)>,
}

impl<G: HasLocations> LocationResolveFilter<G> {
    pub const fn new(min_time: Duration, now: EventTime, location: Location<G>) -> Self {
        Self {
            min_time,
            last_location: (now, location),
            actual_location: None,
        }
    }

    pub const fn new_seeded(min_time: Duration, now: EventTime, location: Location<G>) -> Self {
        Self {
            min_time,
            last_location: (now, location),
            actual_location: Some((now, location)),
        }
    }

    pub fn actual_location(&self) -> Option<Location<G>> {
        self.actual_location.map(|pair| pair.1)
    }

    pub fn update_location(&mut self, now: EventTime, location: Location<G>) -> bool {
        if self.last_location.1 != location {
            self.last_location = (now, location);
            false
        } else {
            if self.actual_location.is_some_and(|pair| pair.1 == location) {
                return false;
            }

            if now.play_time_between(&self.last_location.0) >= self.min_time {
                self.actual_location = Some(self.last_location);
                true
            } else {
                false
            }
        }
    }
}
