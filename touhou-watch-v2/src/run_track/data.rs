use std::fmt::Debug;
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize};
use touhou::memory::RunData;
use touhou::{Difficulty, Location, ShotType, Stage};

use super::{GameSpecificEvent, GameSpecificState};
use crate::time::{GameTime, GameTimeCounter};
use crate::watcher::TrackedGame;

// helper trait for deserialize function
trait SortKey {
    type Key: Ord;

    fn key(&self) -> Self::Key;
}

fn deserialize_sorted_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    Vec<T>: Deserialize<'de>,
    T: SortKey,
{
    <Vec<T> as Deserialize<'de>>::deserialize(deserializer).map(|mut v| {
        v.sort_by_key(|elem| elem.key());
        v
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum EventType<G: TrackedGame> {
    Miss,
    Bomb,
    Pause,
    Unpause,
    Continue,
    GameSpecific(G::Event),
}

impl<G: TrackedGame> EventType<G> {
    pub fn fails_segment(&self) -> bool {
        match self {
            EventType::Miss | EventType::Bomb | EventType::Continue => true,
            EventType::Pause | EventType::Unpause => false,
            EventType::GameSpecific(data) => data.fails_segment(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SegmentEvent<G: TrackedGame> {
    time: GameTime,
    #[serde(rename = "event", alias = "data")]
    data: EventType<G>,
}

impl<G: TrackedGame> SegmentEvent<G> {
    pub(super) fn new(time: GameTime, data: EventType<G>) -> Self {
        Self { time, data }
    }

    pub fn time(&self) -> GameTime {
        self.time
    }

    pub fn data(&self) -> &EventType<G> {
        &self.data
    }

    pub fn fails_segment(&self) -> bool {
        self.data.fails_segment()
    }
}

impl<G: TrackedGame> Deref for SegmentEvent<G> {
    type Target = EventType<G>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<G: TrackedGame> SortKey for SegmentEvent<G> {
    type Key = Self;

    fn key(&self) -> Self::Key {
        *self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSegment<G: TrackedGame> {
    location: Location<G>,
    time: GameTime,
    start_state: G::SegmentState,
    success: bool,
    #[serde(deserialize_with = "deserialize_sorted_vec")]
    events: Vec<SegmentEvent<G>>,
}

impl<G: TrackedGame> StageSegment<G> {
    pub(super) fn new(location: Location<G>, time: GameTime, start_state: G::SegmentState) -> Self {
        Self {
            location,
            start_state,
            time,
            success: true,
            events: Vec::new(),
        }
    }

    pub fn time(&self) -> GameTime {
        self.time
    }

    pub fn location(&self) -> Location<G> {
        self.location
    }

    pub fn start_state(&self) -> &G::SegmentState {
        &self.start_state
    }

    pub fn events(&self) -> &[SegmentEvent<G>] {
        &self.events[..]
    }

    pub fn success(&self) -> bool {
        self.success
    }

    pub(super) fn set_success(&mut self, value: bool) {
        self.success = value;
    }

    pub(super) fn push_event(&mut self, event: SegmentEvent<G>) {
        if event.fails_segment() {
            self.success = false;
        }

        let index = self
            .events
            .binary_search(&event)
            .unwrap_or_else(std::convert::identity);
        self.events.insert(index, event);
    }
}

impl<G: TrackedGame> SortKey for StageSegment<G> {
    type Key = GameTime;

    fn key(&self) -> Self::Key {
        self.time
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStage<G: TrackedGame> {
    stage: Stage<G>,
    time: GameTime,
    #[serde(deserialize_with = "deserialize_sorted_vec")]
    segments: Vec<StageSegment<G>>,
}

impl<G: TrackedGame> RunStage<G> {
    pub(super) fn new(stage: Stage<G>, time: GameTime) -> Self {
        Self {
            stage,
            time,
            segments: Vec::new(),
        }
    }

    pub fn time(&self) -> GameTime {
        self.time
    }

    pub fn stage(&self) -> Stage<G> {
        self.stage
    }

    pub fn segments(&self) -> &[StageSegment<G>] {
        &self.segments[..]
    }

    pub(super) fn push_segment(&mut self, segment: StageSegment<G>) {
        if segment.location.stage() != self.stage {
            panic!(
                "attempted to push location {} to run stage segment {}",
                segment.location, self.stage
            );
        }

        let index = self
            .segments
            .binary_search_by_key(&segment.time, |s| s.time)
            .unwrap_or_else(std::convert::identity);
        self.segments.insert(index, segment);
    }
}

impl<G: TrackedGame> SortKey for RunStage<G> {
    type Key = GameTime;

    fn key(&self) -> Self::Key {
        self.time
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum RunType<G: TrackedGame> {
    Full {
        #[serde(deserialize_with = "deserialize_sorted_vec")]
        stages: Vec<RunStage<G>>,
        cleared: bool,
    },
    StagePractice(RunStage<G>),
    SpellPractice(StageSegment<G>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartEnd<G: TrackedGame> {
    time: GameTime,
    state: G::SegmentState,
}

impl<G: TrackedGame> StartEnd<G> {
    pub(super) fn new(time: GameTime, state: G::SegmentState) -> Self {
        Self { time, state }
    }

    pub fn time(&self) -> GameTime {
        self.time
    }

    pub fn state(&self) -> &G::SegmentState {
        &self.state
    }
}

impl<G: TrackedGame> Deref for StartEnd<G> {
    type Target = G::SegmentState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run<G: TrackedGame> {
    shot: ShotType<G>,
    difficulty: Difficulty<G>,
    start: StartEnd<G>,
    end: StartEnd<G>,
    #[serde(flatten)]
    data: RunType<G>,
}

impl<G: TrackedGame> Run<G> {
    pub(super) fn new(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        start: StartEnd<G>,
        end: StartEnd<G>,
        data: RunType<G>,
    ) -> Self {
        Self {
            shot,
            difficulty,
            start,
            end,
            data,
        }
    }

    pub fn shot(&self) -> ShotType<G> {
        self.shot
    }

    pub fn difficulty(&self) -> Difficulty<G> {
        self.difficulty
    }

    pub fn start(&self) -> &StartEnd<G> {
        &self.start
    }

    pub fn end(&self) -> &StartEnd<G> {
        &self.end
    }

    pub fn data(&self) -> &RunType<G> {
        &self.data
    }
}

impl<G: TrackedGame> Deref for Run<G> {
    type Target = RunType<G>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
