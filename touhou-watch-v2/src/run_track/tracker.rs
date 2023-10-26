use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use touhou::memory::SpellState;
use touhou::{Difficulty, Location, ShotPower, ShotType};

use super::data::{StageSegment, StartEnd};
use super::{EventType, SegmentEvent};
use crate::set_track::LocationResolveFilter;
use crate::time::{GameTime, GameTimeCounter};
use crate::watcher::TrackedGame;

mod builder;
mod run;
mod update;

pub use builder::TrackerBuilder;
pub use run::RunEventCollector;
pub use update::StateUpdate;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NotTracked(PhantomData<()>);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentLives(u8);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TotalMisses(u8);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentBombs(u8);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TotalBombsUsed(u8);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentPower<G: TrackedGame>(ShotPower<G>);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ContinuesUsed(u8);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CurrentPause(bool);

mod private {
    use std::marker::PhantomData;

    pub trait Sealed {}

    #[derive(Debug, Clone, Copy)]
    pub struct SealedToken(PhantomData<()>);

    impl SealedToken {
        pub(super) const fn new() -> SealedToken {
            Self(PhantomData)
        }
    }

    impl Default for SealedToken {
        fn default() -> Self {
            Self::new()
        }
    }
}

use private::{Sealed, SealedToken};

pub trait TrackRun<G: TrackedGame>: Sized {
    type UpdateData;
    type Output;

    fn start_full_run(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self>;

    fn start_stage_practice(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self>;

    fn start_spell_practice(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self>;

    fn start_update(&mut self, time: GameTime, data: Self::UpdateData);

    fn push_event(&mut self, time: GameTime, event: EventType<G>);

    fn end_update(&mut self, time: GameTime, new_location: Option<Location<G>>);

    fn finish(self, time: GameTime, cleared: bool) -> Self::Output;
}

#[derive(Debug)]
pub struct TrackerData<
    G: TrackedGame,
    T: TrackRun<G>,
    TrackLives,
    TrackBombs,
    TrackContinues,
    TrackPause,
> {
    time: GameTimeCounter,
    location_filter: LocationResolveFilter<G>,
    track_lives: TrackLives,
    track_bombs: TrackBombs,
    track_continues: TrackContinues,
    track_pause: TrackPause,
    inner: T,
}

impl<G: TrackedGame, T: TrackRun<G>, TrackLives, TrackBombs, TrackContinues, TrackPause>
    TrackerData<G, T, TrackLives, TrackBombs, TrackContinues, TrackPause>
{
    pub fn now(&self) -> GameTime {
        self.time.now()
    }

    fn push_event(&mut self, data: EventType<G>) {
        let now = self.time.now();
        self.inner.push_event(now, data);
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn start_update(
        &mut self,
        data: T::UpdateData,
    ) -> StateUpdate<
        '_,
        G,
        T,
        TrackLives,
        TrackBombs,
        TrackContinues,
        TrackPause,
        NotTracked,
        NotTracked,
        NotTracked,
        NotTracked,
    > {
        StateUpdate::new(self, data)
    }

    pub fn finish(self, cleared: bool) -> T::Output {
        let now = self.time.now();
        self.inner.finish(now, cleared)
    }
}

impl<G: TrackedGame, T: TrackRun<G>, TrackLives, TrackBombs, TrackContinues, TrackPause> Deref
    for TrackerData<G, T, TrackLives, TrackBombs, TrackContinues, TrackPause>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub const fn new_tracker<G: TrackedGame>()
-> TrackerBuilder<G, NotTracked, NotTracked, NotTracked, NotTracked> {
    TrackerBuilder::new()
}
