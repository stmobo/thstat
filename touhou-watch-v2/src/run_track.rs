use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod data;
mod tracker;

pub use data::{EventType, Run, RunStage, RunType, SegmentEvent, StageSegment, StartEnd};
pub use tracker::{
    new_tracker, ContinuesUsed, CurrentBombs, CurrentLives, CurrentPause, CurrentPower, NotTracked,
    RunEventCollector, StateUpdate, TotalBombsUsed, TotalMisses, TrackRun, TrackerBuilder,
    TrackerData,
};

pub trait GameSpecificEvent: Debug + Copy + Eq + Ord + Serialize + DeserializeOwned {
    fn fails_segment(&self) -> bool;
}

pub trait GameSpecificState: Sized + Clone + Debug + Serialize + DeserializeOwned {}

impl GameSpecificEvent for () {
    fn fails_segment(&self) -> bool {
        false
    }
}

impl GameSpecificState for () {}
