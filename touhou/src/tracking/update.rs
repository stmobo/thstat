//! Logic for updating an active game tracker.

#![allow(clippy::type_complexity)]

use std::marker::PhantomData;

use super::state::*;
use super::tracker::{TrackGame, TrackRun, TrackSpellPractice, TrackStagePractice, UpdateTracker};
use super::{Event, EventTime, GameTimeCounter, TrackableGame, TrackerState};
use crate::memory::traits::{
    BombCount, BombStock, ContinueCount, LifeStock, MissCount, PauseState, PlayerData,
};
use crate::memory::ResolveLocation;
use crate::tracking::TrackingType;
use crate::Location;

impl<G: TrackableGame, T: TrackGame<G>, L, B, C, P> TrackerState<G, T, L, B, C, P> {
    pub fn begin_update(
        &mut self,
        state: G::State,
    ) -> TrackerUpdate<'_, G, T, L, NotTracked, B, NotTracked, C, NotTracked, P, NotTracked> {
        let now = self.now();
        let location_filter = &mut self.location_filter;
        let update = self.tracker.begin_update(now, state);
        let ignore_location_update = self.track_type == TrackingType::SpellPractice;

        TrackerUpdate {
            marker: PhantomData,
            location_filter,
            update,
            miss: false,
            updated_location: CheckOnDrop(
                ignore_location_update,
                "attempted to drop update without location update",
            ),
            finished: CheckOnDrop(false, "attempted to drop unfinished update"),
            lives: &mut self.lives,
            bombs: &mut self.bombs,
            continues: &mut self.continues,
            pause: &mut self.pause,
            time: &mut self.time,
            now,
        }
    }

    pub fn begin_update_with_location<R: ResolveLocation<G>>(
        &mut self,
        state: G::State,
        resolver: &R,
    ) -> TrackerUpdate<'_, G, T, L, NotTracked, B, NotTracked, C, NotTracked, P, NotTracked> {
        let mut update = self.begin_update(state);
        update.update_location(resolver);
        update
    }
}

impl<G: TrackableGame, T: TrackRun<G>, L, B, C, P> TrackerState<G, T, L, B, C, P> {
    pub fn run_cleared(self, state: G::State) -> T::Output {
        assert_eq!(
            self.track_type,
            TrackingType::FullRun,
            "attempted to finish run with incorrect type {:?}",
            self.track_type
        );
        let time = self.now();
        self.tracker.run_cleared(time, state)
    }

    pub fn run_exited(self, state: G::State) -> T::Output {
        assert_eq!(
            self.track_type,
            TrackingType::FullRun,
            "attempted to finish run with incorrect type {:?}",
            self.track_type
        );
        let time = self.now();
        self.tracker.run_exited(time, state)
    }
}

impl<G: TrackableGame, T: TrackStagePractice<G>, L, B, C, P> TrackerState<G, T, L, B, C, P> {
    pub fn finish_stage_practice(self, state: G::State) -> T::Output {
        assert_eq!(
            self.track_type,
            TrackingType::StagePractice,
            "attempted to finish stage practice with incorrect type {:?}",
            self.track_type
        );
        let time = self.now();
        self.tracker.finish_stage_practice(time, state)
    }
}

impl<G: TrackableGame, T: TrackSpellPractice<G>, L, B, C, P> TrackerState<G, T, L, B, C, P> {
    pub fn finish_spell_practice(self, state: G::State) -> T::Output {
        assert_eq!(
            self.track_type,
            TrackingType::SpellPractice,
            "attempted to finish spell practice with incorrect type {:?}",
            self.track_type
        );
        let time = self.now();
        self.tracker.finish_spell_practice(time, state)
    }
}

#[derive(Debug)]
struct CheckOnDrop(bool, &'static str);

impl Drop for CheckOnDrop {
    fn drop(&mut self) {
        if !std::thread::panicking() && !self.0 {
            panic!("{}", self.0);
        }
    }
}

/// Used to update a [`TrackerState`] instance.
///
/// This type is returned from [`TrackerState::begin_update`] and can be used to
/// push events to a running tracker.
///
/// Each update must update all of the state required for misses,
/// bombs, continues, and pausing as determined when the state was initially constructed
/// using [`TrackerBuilder`](super::builder::TrackerBuilder).
///
/// The [`finish`](TrackerUpdate::finish) method can be called once all state has been updated,
/// and it must be called at some point, otherwise this type will panic on drop.
///
/// Tracking for full runs and stage practice must update the tracked location
/// state, and this type will also panic on drop if this is not done.
/// Tracking for spell practice can ignore this requirement, however.
///
/// Note that this type is mainly intended for use by game-specific driver code.
#[must_use = "this will panic on drop if finish() is not called"]
#[derive(Debug)]
pub struct TrackerUpdate<'a, G: TrackableGame, T: TrackGame<G> + 'a, L1, L2, B1, B2, C1, C2, P1, P2>
{
    update: T::Update<'a>,
    location_filter: &'a mut LocationResolveFilter<G>,
    now: EventTime,
    miss: bool,
    finished: CheckOnDrop,
    updated_location: CheckOnDrop,
    lives: &'a mut L1,
    bombs: &'a mut B1,
    continues: &'a mut C1,
    pause: &'a mut P1,
    time: &'a mut GameTimeCounter,
    marker: PhantomData<(L2, B2, C2, P2)>,
}

impl<'a, G, T, L1, L2, B1, B2, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, L1, L2, B1, B2, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn push_event(&mut self, event: Event<G>) {
        self.update.push_event(event);
    }

    pub fn push_game_specific_event(&mut self, event: G::Event) {
        self.update.push_event(Event::GameSpecific(event));
    }

    pub fn now(&self) -> EventTime {
        self.now
    }

    pub fn start_time(&self) -> EventTime {
        self.time.start_time()
    }

    pub fn location(&self) -> Option<Location<G>> {
        self.location_filter.actual_location()
    }

    pub fn update_location<R: ResolveLocation<G>>(&mut self, resolver: &R) {
        if let Some(location) = resolver.resolve_location() {
            if self.location_filter.update_location(self.now, location) {
                if let Some(actual) = self.location_filter.actual_location() {
                    self.update.change_location(Some(actual));
                }
            }
        }

        self.updated_location.0 = true;
    }

    pub fn exit_location(&mut self) {
        self.update.change_location(None);
        self.updated_location.0 = true;
    }
}

impl<'a, G, T, B1, B2, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, CurrentLives, NotTracked, B1, B2, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_life_stock<S: LifeStock<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, CurrentLives, CurrentLives, B1, B2, C1, C2, P1, P2> {
        let miss = self.lives.update(state);
        if miss {
            self.push_event(Event::Miss);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss,
        }
    }
}

impl<'a, G, T, B1, B2, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, TotalMisses, NotTracked, B1, B2, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_total_misses<S: MissCount<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, TotalMisses, TotalMisses, B1, B2, C1, C2, P1, P2> {
        let miss = self.lives.update(state);
        if miss {
            self.push_event(Event::Miss);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss,
        }
    }
}

impl<'a, G, T, L1, L2, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, L1, L2, CurrentBombs, NotTracked, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_bomb_stock<S: BombStock<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, L1, L2, CurrentBombs, CurrentBombs, C1, C2, P1, P2> {
        let bombed = self.bombs.update(state);
        if bombed {
            self.push_event(Event::Bomb);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss: self.miss,
        }
    }
}

impl<'a, G, T, L1, L2, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, L1, L2, TotalBombsUsed, NotTracked, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_total_bombs_used<S: BombCount<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, L1, L2, TotalBombsUsed, TotalBombsUsed, C1, C2, P1, P2> {
        let bombed = self.bombs.update(state);
        if bombed {
            self.push_event(Event::Bomb);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss: self.miss,
        }
    }
}

impl<'a, G, T, L, C1, C2, P1, P2>
    TrackerUpdate<'a, G, T, L, L, CurrentPower<G>, NotTracked, C1, C2, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_power<S: PlayerData<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, L, L, CurrentPower<G>, CurrentPower<G>, C1, C2, P1, P2> {
        let bombed = self.bombs.update(state, self.miss);
        if bombed {
            self.push_event(Event::Bomb);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss: self.miss,
        }
    }
}

impl<'a, G, T, L1, L2, B1, B2, P1, P2>
    TrackerUpdate<'a, G, T, L1, L2, B1, B2, ContinuesUsed, NotTracked, P1, P2>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_continues_used<S: ContinueCount<G>>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, L1, L2, B1, B2, ContinuesUsed, ContinuesUsed, P1, P2> {
        let used_continue = self.continues.update(state);
        if used_continue {
            self.push_event(Event::Continue);
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss: self.miss,
        }
    }
}

impl<'a, G, T, L1, L2, B1, B2, C1, C2>
    TrackerUpdate<'a, G, T, L1, L2, B1, B2, C1, C2, CurrentPause, NotTracked>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn update_pause<S: PauseState>(
        mut self,
        state: &S,
    ) -> TrackerUpdate<'a, G, T, L1, L2, B1, B2, C1, C2, CurrentPause, CurrentPause> {
        let new_pause = CurrentPause::new(state);
        let prev_pause = std::mem::replace(self.pause, new_pause);
        match (prev_pause.is_paused(), new_pause.is_paused()) {
            (false, true) => {
                self.push_event(Event::Pause);
                self.time.pause();
            }
            (true, false) => {
                self.push_event(Event::Unpause);
                self.time.unpause();
            }
            (false, false) | (true, true) => {}
        }

        TrackerUpdate {
            marker: PhantomData,
            update: self.update,
            lives: self.lives,
            bombs: self.bombs,
            pause: self.pause,
            continues: self.continues,
            location_filter: self.location_filter,
            time: self.time,
            now: self.now,
            updated_location: self.updated_location,
            finished: self.finished,
            miss: self.miss,
        }
    }
}

impl<'a, G, T, L, B, C, P> TrackerUpdate<'a, G, T, L, L, B, B, C, C, P, P>
where
    G: TrackableGame,
    T: TrackGame<G>,
{
    pub fn finish(mut self) {
        self.finished.0 = true;
    }
}
