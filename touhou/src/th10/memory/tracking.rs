use std::ops::Deref;
use std::time::Duration;

use super::process::MemoryAccess;
use super::{GameMemory, GameState, RunState};
use crate::memory::{MemoryReadError, PlayerData};
use crate::tracking::builder::TrackerBuilder;
use crate::tracking::state::{ContinuesUsed, CurrentLives, CurrentPower, NotTracked};
use crate::tracking::{
    DriveTracker, GameTracker, IntoGameTracker, TrackRun, TrackStagePractice, TrackableGame,
    TrackerState, TrackingType, UpdateStatus,
};
use crate::Touhou10;

impl TrackableGame for Touhou10 {
    type State = RunState;
    type Event = ();
}

#[derive(Debug)]
pub struct ActiveRun<T> {
    tracker:
        TrackerState<Touhou10, T, CurrentLives, CurrentPower<Touhou10>, ContinuesUsed, NotTracked>,
    prev_state: RunState,
}

impl<T> ActiveRun<T>
where
    T: TrackRun<Touhou10> + TrackStagePractice<Touhou10>,
{
    fn new(state: RunState) -> Self {
        let player = state.player();
        let builder = TrackerBuilder::new()
            .track_life_stock(&player)
            .track_power(&player)
            .track_continues(&player);

        let tracker = if state.practice() {
            builder.start_stage_practice(
                player.shot(),
                state.difficulty(),
                state.stage().stage(),
                state,
                Duration::from_millis(750),
            )
        } else {
            builder.start_run(
                player.shot(),
                state.difficulty(),
                state,
                Duration::from_millis(750),
            )
        };

        Self {
            tracker,
            prev_state: state,
        }
    }

    fn update_state(&mut self, state: RunState) {
        let player = state.player();
        self.tracker
            .begin_update_with_location(state, &state)
            .update_life_stock(&player)
            .update_power(&player)
            .update_continues_used(&player)
            .finish();
        self.prev_state = state;
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

impl<T> DriveTracker<Touhou10, T> for ActiveRun<T>
where
    T: TrackRun<Touhou10> + TrackStagePractice<Touhou10>,
{
    type Memory = GameMemory;

    fn game_is_active(access: &MemoryAccess) -> Result<bool, MemoryReadError<Touhou10>> {
        GameState::game_is_active(access)
    }

    fn init(access: &MemoryAccess) -> Result<Option<Self>, MemoryReadError<Touhou10>> {
        GameState::new(access).map(|state| {
            if let GameState::InGame(run) = state {
                Some(Self::new(run))
            } else {
                None
            }
        })
    }

    fn update(
        mut self,
        access: &MemoryAccess,
    ) -> Result<UpdateStatus<Touhou10, T, Self>, MemoryReadError<Touhou10>> {
        match GameState::new(access)? {
            GameState::InGame(run) => {
                self.update_state(run);
                Ok(UpdateStatus::Continuing(self))
            }
            GameState::GameOver(run) => Ok(UpdateStatus::Finished(self.finish(false, Some(run)))),
            GameState::Ending(run) => Ok(UpdateStatus::Finished(self.finish(true, Some(run)))),
            _ => Ok(UpdateStatus::Finished(self.finish(false, None))),
        }
    }

    fn terminate(self) -> T::Output {
        self.finish(false, None)
    }
}

impl<T> GameTracker<Touhou10, T, ActiveRun<T>>
where
    T: TrackRun<Touhou10> + TrackStagePractice<Touhou10>,
{
    pub fn new_th10(memory: GameMemory) -> Self {
        Self::new(memory)
    }
}

impl<T> IntoGameTracker<Touhou10, T> for GameMemory
where
    T: TrackRun<Touhou10> + TrackStagePractice<Touhou10>,
    ActiveRun<T>: DriveTracker<Touhou10, T, Memory = GameMemory>,
{
    type Driver = ActiveRun<T>;

    fn track_games(self) -> GameTracker<Touhou10, T, ActiveRun<T>> {
        GameTracker::new(self)
    }
}
