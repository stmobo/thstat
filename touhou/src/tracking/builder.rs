//! Logic for building new [`TrackerState`] instances.

use std::marker::PhantomData;
use std::time::Duration;

use super::state::*;
use super::tracker::{TrackRun, TrackSpellPractice, TrackStagePractice};
use super::{EventTime, GameTimeCounter, TrackableGame, TrackerState, TrackingType};
use crate::memory::traits::{
    BombCount, BombStock, ContinueCount, LifeStock, MissCount, PauseState, PlayerData,
};
use crate::{Difficulty, Location, ShotType, Stage};

/// Constructs a new [`TrackerState`] instance.
///
/// The chainable methods on this type can be used to specify data sources for
/// standard events (misses, bombs, continues, and pausing) for a new tracker.
///
/// Once set up, you can then call one of [`start_run`], [`start_stage_practice`], or
/// [`start_spell_practice`] to begin tracking a game.
///
/// Note that this type is primarily meant to be used by game-specific driver code.
pub struct TrackerBuilder<G, L, B, C, P> {
    marker: PhantomData<G>,
    lives: L,
    bombs: B,
    continues: C,
    pause: P,
    time: Option<GameTimeCounter>,
}

impl<G: TrackableGame> TrackerBuilder<G, NotTracked, NotTracked, NotTracked, NotTracked> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
            lives: NotTracked::new(),
            bombs: NotTracked::new(),
            continues: NotTracked::new(),
            pause: NotTracked::new(),
            time: None,
        }
    }
}

impl<G: TrackableGame> Default
    for TrackerBuilder<G, NotTracked, NotTracked, NotTracked, NotTracked>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<G: TrackableGame, B, C, P> TrackerBuilder<G, NotTracked, B, C, P> {
    pub fn track_life_stock<S: LifeStock<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, CurrentLives, B, C, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: CurrentLives::new(state),
            bombs: self.bombs,
            continues: self.continues,
            pause: self.pause,
            time: self.time,
        }
    }

    pub fn track_total_misses<S: MissCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, TotalMisses, B, C, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: TotalMisses::new(state),
            bombs: self.bombs,
            continues: self.continues,
            pause: self.pause,
            time: self.time,
        }
    }
}

impl<G: TrackableGame, L, C, P> TrackerBuilder<G, L, NotTracked, C, P> {
    pub fn track_bomb_stock<S: BombStock<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, L, CurrentBombs, C, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: self.lives,
            bombs: CurrentBombs::new(state),
            continues: self.continues,
            pause: self.pause,
            time: self.time,
        }
    }

    pub fn track_total_bombs_used<S: BombCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, L, TotalBombsUsed, C, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: self.lives,
            bombs: TotalBombsUsed::new(state),
            continues: self.continues,
            pause: self.pause,
            time: self.time,
        }
    }

    pub fn track_power<S: PlayerData<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, L, CurrentPower<G>, C, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: self.lives,
            bombs: CurrentPower::new(state),
            continues: self.continues,
            pause: self.pause,
            time: self.time,
        }
    }
}

impl<G: TrackableGame, L, B, P> TrackerBuilder<G, L, B, NotTracked, P> {
    pub fn track_continues<S: ContinueCount<G>>(
        self,
        state: &S,
    ) -> TrackerBuilder<G, L, B, ContinuesUsed, P> {
        TrackerBuilder {
            marker: PhantomData,
            lives: self.lives,
            bombs: self.bombs,
            continues: ContinuesUsed::new(state),
            pause: self.pause,
            time: self.time,
        }
    }
}

impl<G: TrackableGame, L, B, C> TrackerBuilder<G, L, B, C, NotTracked> {
    pub fn track_pause<S: PauseState>(self, state: &S) -> TrackerBuilder<G, L, B, C, CurrentPause> {
        TrackerBuilder {
            marker: PhantomData,
            lives: self.lives,
            bombs: self.bombs,
            continues: self.continues,
            pause: CurrentPause::new(state),
            time: Some(GameTimeCounter::new(state.paused())),
        }
    }
}

impl<G, L, B, C, P> TrackerBuilder<G, L, B, C, P>
where
    G: TrackableGame,
    L: TrackLives,
    B: TrackBombs,
    C: TrackContinues,
    P: TrackPause,
{
    pub fn start_time(&mut self) -> EventTime {
        if self.time.is_none() {
            self.time = Some(GameTimeCounter::new(self.pause.is_paused()));
        }

        self.time.as_ref().unwrap().start_time()
    }

    pub fn start_run<T: TrackRun<G>>(
        self,
        shot_type: ShotType<G>,
        difficulty: Difficulty<G>,
        state: G::State,
        min_location_time: Duration,
    ) -> TrackerState<G, T, L, B, C, P> {
        let time = GameTimeCounter::new(self.pause.is_paused());
        let now = time.start_time();
        let location_filter =
            LocationResolveFilter::new(min_location_time, now, Location::default());
        let tracker = T::start_run(now, shot_type, difficulty, state);

        TrackerState {
            time,
            location_filter,
            track_type: TrackingType::FullRun,
            tracker,
            lives: self.lives,
            bombs: self.bombs,
            continues: self.continues,
            pause: self.pause,
        }
    }

    pub fn start_stage_practice<T: TrackStagePractice<G>>(
        self,
        shot_type: ShotType<G>,
        difficulty: Difficulty<G>,
        stage: Stage<G>,
        state: G::State,
        min_location_time: Duration,
    ) -> TrackerState<G, T, L, B, C, P> {
        let time = GameTimeCounter::new(self.pause.is_paused());
        let now = time.start_time();
        let location_filter =
            LocationResolveFilter::new(min_location_time, now, stage.start_location());
        let tracker = T::start_stage_practice(now, shot_type, difficulty, stage, state);

        TrackerState {
            time,
            location_filter,
            track_type: TrackingType::StagePractice,
            tracker,
            lives: self.lives,
            bombs: self.bombs,
            continues: self.continues,
            pause: self.pause,
        }
    }

    pub fn start_spell_practice<T: TrackSpellPractice<G>>(
        self,
        shot_type: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        state: G::State,
        min_location_time: Duration,
    ) -> TrackerState<G, T, L, B, C, P> {
        let time = GameTimeCounter::new(self.pause.is_paused());
        let now = time.start_time();
        let location_filter = LocationResolveFilter::new_seeded(min_location_time, now, location);
        let tracker = T::start_spell_practice(now, shot_type, difficulty, location, state);

        TrackerState {
            time,
            location_filter,
            track_type: TrackingType::SpellPractice,
            tracker,
            lives: self.lives,
            bombs: self.bombs,
            continues: self.continues,
            pause: self.pause,
        }
    }
}
