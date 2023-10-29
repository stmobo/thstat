use std::ops::Deref;
use std::time::Duration;

use super::process::MemoryAccess;
use super::{GameMemory, GameState, RunState};
use crate::memory::{MemoryReadError, PlayerData};
use crate::tracking::builder::TrackerBuilder;
use crate::tracking::state::{ContinuesUsed, CurrentPause, TotalBombsUsed, TotalMisses};
use crate::tracking::{
    DriveTracker, EventTime, GameTracker, IntoGameTracker, TrackGame, TrackRun, TrackStagePractice,
    TrackableGame, TrackerState, TrackingType, UpdateStatus,
};
use crate::Touhou7;

#[derive(Debug, Clone, Copy)]
pub struct TrackedState {
    state: RunState,
    border_start_time: Option<EventTime>,
}

impl TrackedState {
    pub fn border_start_time(&self) -> Option<EventTime> {
        self.border_start_time
    }

    pub fn new(state: RunState, border_start_time: Option<EventTime>) -> Self {
        Self {
            state,
            border_start_time,
        }
    }
}

impl AsRef<RunState> for TrackedState {
    fn as_ref(&self) -> &RunState {
        &self.state
    }
}

impl Deref for TrackedState {
    type Target = RunState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Touhou7Event {
    BorderStart,
    BorderEnd { broken: bool },
}

impl std::fmt::Display for Touhou7Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BorderStart => "Border Start",
            Self::BorderEnd { broken: true } => "Border Break",
            Self::BorderEnd { broken: false } => "Border End",
        }
        .fmt(f)
    }
}

impl TrackableGame for Touhou7 {
    type State = TrackedState;
    type Event = Touhou7Event;
}

enum BorderChange {
    BorderStart,
    BorderEnd(Duration),
    NoChange,
}

#[derive(Debug)]
pub struct ActiveRun<T> {
    tracker: TrackerState<Touhou7, T, TotalMisses, TotalBombsUsed, ContinuesUsed, CurrentPause>,
    prev_state: TrackedState,
}

impl<T> ActiveRun<T>
where
    T: TrackRun<Touhou7> + TrackStagePractice<Touhou7>,
{
    fn new(state: RunState) -> Self {
        let player = state.player();
        let mut builder = TrackerBuilder::new()
            .track_total_misses(&player)
            .track_total_bombs_used(&player)
            .track_continues(&player)
            .track_pause(&state);

        let border_start_time = if player.border_active() {
            Some(builder.start_time())
        } else {
            None
        };
        let tracked_state = TrackedState::new(state, border_start_time);

        let tracker = if state.practice() {
            builder.start_stage_practice(
                player.shot(),
                state.difficulty(),
                state.stage().stage(),
                tracked_state,
                Duration::from_millis(750),
            )
        } else {
            builder.start_run(
                player.shot(),
                state.difficulty(),
                tracked_state,
                Duration::from_millis(750),
            )
        };

        Self {
            tracker,
            prev_state: tracked_state,
        }
    }

    fn update_state(&mut self, state: RunState) {
        let player = state.player();
        let now = self.tracker.now();

        let (border_change, new_state) =
            match (self.prev_state.border_start_time, player.border_active()) {
                (None, true) => (
                    BorderChange::BorderStart,
                    TrackedState::new(state, Some(now)),
                ),
                (Some(prev_start), false) => (
                    BorderChange::BorderEnd(prev_start.play_time_between(&now)),
                    TrackedState::new(state, None),
                ),
                (Some(_), true) | (None, false) => (
                    BorderChange::NoChange,
                    TrackedState::new(state, self.prev_state.border_start_time),
                ),
            };

        let mut update = self.tracker.begin_update(new_state);
        update.update_location(&state);

        match border_change {
            BorderChange::BorderStart => {
                update.push_game_specific_event(Touhou7Event::BorderStart);
            }
            BorderChange::BorderEnd(duration) => {
                update.push_game_specific_event(Touhou7Event::BorderEnd {
                    broken: duration >= Duration::from_millis(8750),
                })
            }
            BorderChange::NoChange => {}
        }

        update
            .update_total_misses(&player)
            .update_total_bombs_used(&player)
            .update_continues_used(&player)
            .update_pause(&state)
            .finish();

        self.prev_state = new_state;
    }

    fn finish(mut self, cleared: bool, end_state: Option<RunState>) -> T::Output {
        if let Some(end_state) = end_state {
            self.update_state(end_state);
        }

        if self.tracker.tracking_type() == TrackingType::StagePractice {
            self.tracker.finish_stage_practice(self.prev_state)
        } else if cleared {
            self.tracker.run_cleared(self.prev_state)
        } else {
            self.tracker.run_exited(self.prev_state)
        }
    }
}

impl<T> DriveTracker<Touhou7, T> for ActiveRun<T>
where
    T: TrackRun<Touhou7> + TrackStagePractice<Touhou7>,
{
    type Memory = GameMemory;

    fn game_is_active(access: &MemoryAccess) -> Result<bool, MemoryReadError<Touhou7>> {
        GameState::game_is_active(access)
    }

    fn init(access: &MemoryAccess) -> Result<Option<Self>, MemoryReadError<Touhou7>> {
        GameState::new(access).map(|state| {
            if let GameState::InGame { run } = state {
                Some(Self::new(run))
            } else {
                None
            }
        })
    }

    fn update(
        mut self,
        access: &MemoryAccess,
    ) -> Result<UpdateStatus<Touhou7, T, Self>, MemoryReadError<Touhou7>> {
        match GameState::new(access)? {
            GameState::InGame { run } => {
                self.update_state(run);
                Ok(UpdateStatus::Continuing(self))
            }
            GameState::LoadingStage => Ok(UpdateStatus::Continuing(self)),
            GameState::GameOver { cleared, run } => {
                Ok(UpdateStatus::Finished(self.finish(cleared, Some(run))))
            }
            _ => Ok(UpdateStatus::Finished(self.finish(false, None))),
        }
    }

    fn terminate(self) -> T::Output {
        self.finish(false, None)
    }
}

impl<T> GameTracker<Touhou7, T, ActiveRun<T>>
where
    T: TrackRun<Touhou7> + TrackStagePractice<Touhou7>,
{
    pub fn new_th07(memory: GameMemory) -> Self {
        Self::new(memory)
    }
}

impl<T> IntoGameTracker<Touhou7, T> for GameMemory
where
    T: TrackRun<Touhou7> + TrackStagePractice<Touhou7>,
    ActiveRun<T>: DriveTracker<Touhou7, T, Memory = GameMemory>,
{
    type Driver = ActiveRun<T>;

    fn track_games(self) -> GameTracker<Touhou7, T, ActiveRun<T>> {
        GameTracker::new(self)
    }
}
