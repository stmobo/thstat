use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use touhou::memory::traits::{
    BombCount, BombStock, LifeStock, MissCount, PlayerData, ResolveLocation, RunData, StageData,
};
use touhou::memory::{ContinueCount, PauseState};

use super::super::data::{EventType, StageSegment};
use super::{
    ContinuesUsed, CurrentBombs, CurrentLives, CurrentPause, CurrentPower, NotTracked, SealedToken,
    TotalBombsUsed, TotalMisses, TrackRun, TrackerData,
};
use crate::watcher::TrackedGame;

#[derive(Debug)]
pub struct StateUpdate<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, C1, P1, L2, B2, C2, P2> {
    data: &'a mut TrackerData<G, T, L1, B1, C1, P1>,
    emitted_miss: bool,
    marker: PhantomData<(L2, B2, C2, P2)>,
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, C1, P1, L2, B2, C2, P2> Deref
    for StateUpdate<'a, G, T, L1, B1, C1, P1, L2, B2, C2, P2>
{
    type Target = TrackerData<G, T, L1, B1, C1, P1>;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, C1, P1, L2, B2, C2, P2> DerefMut
    for StateUpdate<'a, G, T, L1, B1, C1, P1, L2, B2, C2, P2>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L, B, C, P>
    StateUpdate<'a, G, T, L, B, C, P, NotTracked, NotTracked, NotTracked, NotTracked>
{
    pub(super) fn new(
        data: &'a mut TrackerData<G, T, L, B, C, P>,
        update_data: T::UpdateData,
    ) -> Self {
        data.inner.start_update(data.now(), update_data);

        Self {
            data,
            emitted_miss: false,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, C1, P1, L2, B2, C2, P2>
    StateUpdate<'a, G, T, L1, B1, C1, P1, L2, B2, C2, P2>
{
    pub fn push_game_event(&mut self, event: G::Event) {
        self.push_event(EventType::GameSpecific(event))
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, B1, C1, P1, B2, C2, P2>
    StateUpdate<'a, G, T, CurrentLives, B1, C1, P1, NotTracked, B2, C2, P2>
{
    pub fn update_life_stock<S: LifeStock<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, CurrentLives, B1, C1, P1, CurrentLives, B2, C2, P2> {
        let new_lives = state.lives();
        let prev_lives = self.track_lives.0;
        let now = self.time.now();

        if new_lives < prev_lives {
            self.push_event(EventType::Miss);
            self.emitted_miss = true;
        }

        self.track_lives = CurrentLives(new_lives);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, B1, C1, P1, B2, C2, P2>
    StateUpdate<'a, G, T, TotalMisses, B1, C1, P1, NotTracked, B2, C2, P2>
{
    pub fn update_total_misses<S: MissCount<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, TotalMisses, B1, C1, P1, TotalMisses, B2, C2, P2> {
        let new_misses = state.total_misses();
        let prev_misses = self.track_lives.0;
        let now = self.time.now();

        if new_misses > prev_misses {
            self.push_event(EventType::Miss);
            self.emitted_miss = true;
        }

        self.track_lives = TotalMisses(new_misses);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, C1, P1, L2, C2, P2>
    StateUpdate<'a, G, T, L1, CurrentBombs, C1, P1, L2, NotTracked, C2, P2>
{
    pub fn update_bomb_stock<S: BombStock<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, L1, CurrentBombs, C1, P1, L2, CurrentBombs, C2, P2> {
        let new_stock = state.bombs();
        let prev_stock = self.track_bombs.0;
        let now = self.time.now();

        if new_stock < prev_stock {
            self.push_event(EventType::Bomb);
        }

        self.track_bombs = CurrentBombs(new_stock);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, C1, P1, L2, C2, P2>
    StateUpdate<'a, G, T, L1, TotalBombsUsed, C1, P1, L2, NotTracked, C2, P2>
{
    pub fn update_total_bombs<S: BombCount<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, L1, TotalBombsUsed, C1, P1, L2, TotalBombsUsed, C2, P2> {
        let new_total = state.total_bombs();
        let prev_total = self.track_bombs.0;
        let now = self.time.now();

        if new_total > prev_total {
            self.push_event(EventType::Bomb);
        }

        self.track_bombs = TotalBombsUsed(new_total);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, C1, P1, C2, P2>
    StateUpdate<'a, G, T, L1, CurrentPower<G>, C1, P1, NotTracked, NotTracked, C2, P2>
{
    pub fn update_power<S: PlayerData<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, L1, CurrentPower<G>, C1, P1, NotTracked, CurrentPower<G>, C2, P2>
    {
        let new_power = state.power();
        let prev_power = self.track_bombs.0;
        let now = self.time.now();

        if (new_power < prev_power) && !self.emitted_miss {
            self.push_event(EventType::Bomb);
        }

        self.track_bombs = CurrentPower(new_power);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, P1, L2, B2, P2>
    StateUpdate<'a, G, T, L1, B1, ContinuesUsed, P1, L2, B2, NotTracked, P2>
{
    pub fn update_continues<S: ContinueCount<G>>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, L1, B1, ContinuesUsed, P1, L2, B2, ContinuesUsed, P2> {
        let new_total = state.continues_used();
        let prev_total = self.track_continues.0;
        let now = self.time.now();

        if new_total > prev_total {
            self.push_event(EventType::Continue);
        }

        self.track_continues = ContinuesUsed(new_total);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L1, B1, C1, L2, B2, C2>
    StateUpdate<'a, G, T, L1, B1, C1, CurrentPause, L2, B2, C2, NotTracked>
{
    pub fn update_pause<S: PauseState>(
        mut self,
        state: &S,
    ) -> StateUpdate<'a, G, T, L1, B1, C1, CurrentPause, L2, B2, C2, CurrentPause> {
        let new_state = state.paused();
        let prev_state = self.track_pause.0;

        match (prev_state, new_state) {
            (true, false) => {
                self.time.unpause();
                self.push_event(EventType::Unpause);
            }
            (false, true) => {
                self.time.pause();
                self.push_event(EventType::Pause);
            }
            _ => {}
        }

        self.track_pause = CurrentPause(new_state);
        StateUpdate {
            data: self.data,
            emitted_miss: self.emitted_miss,
            marker: PhantomData,
        }
    }
}

impl<'a, G: TrackedGame, T: TrackRun<G>, L, B, C, P> StateUpdate<'a, G, T, L, B, C, P, L, B, C, P> {
    pub fn finish<S: RunData<G>, R: ResolveLocation<G>>(mut self, state: &S, resolver: &R) {
        if let Some(location) = resolver.resolve_location() {
            let now = self.time.now();
            let new_location = if self.location_filter.update_location(location) {
                self.location_filter.actual_location()
            } else {
                None
            };

            self.inner.end_update(now, new_location);
        }
    }
}
