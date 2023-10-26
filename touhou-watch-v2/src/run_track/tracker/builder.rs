use std::marker::PhantomData;

use touhou::memory::traits::{
    BombCount, BombStock, LifeStock, MissCount, PlayerData, RunData, StageData,
};
use touhou::memory::{ContinueCount, PauseState};
use touhou::Location;

use super::{
    ContinuesUsed, CurrentBombs, CurrentLives, CurrentPause, CurrentPower, NotTracked,
    TotalBombsUsed, TotalMisses, TrackRun, TrackerData,
};
use crate::set_track::LocationResolveFilter;
use crate::time::{EventTime, GameTimeCounter};
use crate::watcher::TrackedGame;

#[derive(Debug)]
pub struct TrackerBuilder<G: TrackedGame, TrackLives, TrackBombs, TrackContinues, TrackPause> {
    marker: PhantomData<G>,
    track_lives: TrackLives,
    track_bombs: TrackBombs,
    track_continues: TrackContinues,
    track_pause: TrackPause,
}

impl<G: TrackedGame> TrackerBuilder<G, NotTracked, NotTracked, NotTracked, NotTracked> {
    pub(super) const fn new() -> Self {
        Self {
            marker: PhantomData,
            track_lives: NotTracked(PhantomData),
            track_bombs: NotTracked(PhantomData),
            track_continues: NotTracked(PhantomData),
            track_pause: NotTracked(PhantomData),
        }
    }
}

impl<G: TrackedGame, TrackBombs, TrackContinues, TrackPause>
    TrackerBuilder<G, NotTracked, TrackBombs, TrackContinues, TrackPause>
{
    pub fn track_life_stock<S: LifeStock<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, CurrentLives, TrackBombs, TrackContinues, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: CurrentLives(state.lives()),
            track_bombs: self.track_bombs,
            track_continues: self.track_continues,
            track_pause: self.track_pause,
        }
    }

    pub fn track_total_misses<S: MissCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TotalMisses, TrackBombs, TrackContinues, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: TotalMisses(state.total_misses()),
            track_bombs: self.track_bombs,
            track_continues: self.track_continues,
            track_pause: self.track_pause,
        }
    }
}

impl<G: TrackedGame, TrackLives, TrackContinues, TrackPause>
    TrackerBuilder<G, TrackLives, NotTracked, TrackContinues, TrackPause>
{
    pub fn track_bomb_stock<S: BombStock<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TrackLives, CurrentBombs, TrackContinues, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: self.track_lives,
            track_bombs: CurrentBombs(state.bombs()),
            track_continues: self.track_continues,
            track_pause: self.track_pause,
        }
    }

    pub fn track_total_bombs<S: BombCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TrackLives, TotalBombsUsed, TrackContinues, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: self.track_lives,
            track_bombs: TotalBombsUsed(state.total_bombs()),
            track_continues: self.track_continues,
            track_pause: self.track_pause,
        }
    }

    pub fn track_power<S: PlayerData<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TrackLives, CurrentPower<G>, TrackContinues, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: self.track_lives,
            track_bombs: CurrentPower(state.power()),
            track_continues: self.track_continues,
            track_pause: self.track_pause,
        }
    }
}

impl<G: TrackedGame, TrackLives, TrackBombs, TrackPause>
    TrackerBuilder<G, TrackLives, TrackBombs, NotTracked, TrackPause>
{
    pub fn track_continues<S: ContinueCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TrackLives, TrackBombs, ContinuesUsed, TrackPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: self.track_lives,
            track_bombs: self.track_bombs,
            track_continues: ContinuesUsed(state.continues_used()),
            track_pause: self.track_pause,
        }
    }
}

impl<G: TrackedGame, TrackLives, TrackBombs, TrackContinues>
    TrackerBuilder<G, TrackLives, TrackBombs, TrackContinues, NotTracked>
{
    pub fn track_pause<S: PauseState>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TrackLives, TrackBombs, TrackContinues, CurrentPause> {
        TrackerBuilder {
            marker: PhantomData,
            track_lives: self.track_lives,
            track_bombs: self.track_bombs,
            track_continues: self.track_continues,
            track_pause: CurrentPause(state.paused()),
        }
    }
}

impl<G: TrackedGame, TrackLives, TrackBombs, TrackContinues, TrackPause>
    TrackerBuilder<G, TrackLives, TrackBombs, TrackContinues, TrackPause>
{
    pub fn start_full_run<T: TrackRun<G>, S: RunData<G>>(
        self,
        state: &S,
        location: Location<G>,
        data: T::UpdateData,
    ) -> Option<TrackerData<G, T, TrackLives, TrackBombs, TrackContinues, TrackPause>> {
        let stage = state.stage();
        let player = state.player();
        let time = GameTimeCounter::new(EventTime::new());
        let now = time.now();

        assert!(location.spell().is_none());
        assert!(stage.active_spell().is_none());

        T::start_full_run(player.shot(), state.difficulty(), location, now, data).map(|inner| {
            TrackerData {
                time,
                location_filter: LocationResolveFilter::new_seeded(location),
                track_lives: self.track_lives,
                track_bombs: self.track_bombs,
                track_continues: self.track_continues,
                track_pause: self.track_pause,
                inner,
            }
        })
    }

    pub fn start_stage_practice<T: TrackRun<G>, S: RunData<G>>(
        self,
        state: &S,
        location: Location<G>,
        data: T::UpdateData,
    ) -> Option<TrackerData<G, T, TrackLives, TrackBombs, TrackContinues, TrackPause>> {
        let stage = state.stage();
        let player = state.player();
        let time = GameTimeCounter::new(EventTime::new());
        let now = time.now();

        T::start_stage_practice(player.shot(), state.difficulty(), location, now, data).map(
            |inner| TrackerData {
                time,
                location_filter: LocationResolveFilter::new_seeded(location),
                track_lives: self.track_lives,
                track_bombs: self.track_bombs,
                track_continues: self.track_continues,
                track_pause: self.track_pause,
                inner,
            },
        )
    }

    pub fn start_spell_practice<T: TrackRun<G>, S: RunData<G>>(
        self,
        state: &S,
        location: Location<G>,
        data: T::UpdateData,
    ) -> Option<TrackerData<G, T, TrackLives, TrackBombs, TrackContinues, TrackPause>> {
        let stage = state.stage();
        let player = state.player();
        let time = GameTimeCounter::new(EventTime::new());
        let now = time.now();

        T::start_spell_practice(player.shot(), state.difficulty(), location, now, data).map(
            |inner| TrackerData {
                time,
                location_filter: LocationResolveFilter::new_seeded(location),
                track_lives: self.track_lives,
                track_bombs: self.track_bombs,
                track_continues: self.track_continues,
                track_pause: self.track_pause,
                inner,
            },
        )
    }
}
