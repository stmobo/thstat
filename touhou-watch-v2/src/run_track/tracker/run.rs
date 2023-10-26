use touhou::{Difficulty, Location, ShotType, Stage};

use super::super::{RunStage, StageSegment};
use super::{TrackRun, TrackerData};
use crate::run_track::{EventType, Run, RunType, SegmentEvent, StartEnd};
use crate::time::GameTime;
use crate::TrackedGame;

#[derive(Debug)]
enum TrackerMode<G: TrackedGame> {
    Full(Vec<RunStage<G>>),
    StagePractice(RunStage<G>),
    SpellPractice(Option<StageSegment<G>>),
}

impl<G: TrackedGame> TrackerMode<G> {
    fn push_segment(&mut self, prev_segment: StageSegment<G>) {
        match self {
            TrackerMode::Full(stages) => {
                let stage = prev_segment.location().stage();
                if let Some(current_stage) = stages.last_mut().filter(|s| s.stage() == stage) {
                    current_stage.push_segment(prev_segment);
                } else {
                    let mut current_stage =
                        RunStage::new(prev_segment.location().stage(), prev_segment.time());
                    current_stage.push_segment(prev_segment);
                    stages.push(current_stage);
                }
            }
            TrackerMode::StagePractice(stage) => stage.push_segment(prev_segment),
            TrackerMode::SpellPractice(last) => {
                *last = Some(prev_segment);
            }
        }
    }
}

#[derive(Debug)]
pub struct RunEventCollector<G: TrackedGame> {
    mode: TrackerMode<G>,
    start: StartEnd<G>,
    shot: ShotType<G>,
    difficulty: Difficulty<G>,
    last_state: G::SegmentState,
    cur_segment: StageSegment<G>,
}

impl<G: TrackedGame> RunEventCollector<G> {
    fn push_segment(&mut self, new_segment: StageSegment<G>) {
        self.mode
            .push_segment(std::mem::replace(&mut self.cur_segment, new_segment))
    }

    pub fn last_state(&self) -> &G::SegmentState {
        &self.last_state
    }
}

impl<G: TrackedGame> TrackRun<G> for RunEventCollector<G> {
    type Output = Run<G>;
    type UpdateData = G::SegmentState;

    fn start_full_run(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self> {
        Some(Self {
            mode: TrackerMode::Full(Vec::new()),
            start: StartEnd::new(time, data.clone()),
            cur_segment: StageSegment::new(location, time, data.clone()),
            shot,
            difficulty,
            last_state: data,
        })
    }

    fn start_stage_practice(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self> {
        Some(Self {
            mode: TrackerMode::StagePractice(RunStage::new(location.stage(), time)),
            start: StartEnd::new(time, data.clone()),
            cur_segment: StageSegment::new(location, time, data.clone()),
            shot,
            difficulty,
            last_state: data,
        })
    }

    fn start_spell_practice(
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        time: GameTime,
        data: Self::UpdateData,
    ) -> Option<Self> {
        Some(Self {
            mode: TrackerMode::SpellPractice(None),
            start: StartEnd::new(time, data.clone()),
            cur_segment: StageSegment::new(location, time, data.clone()),
            shot,
            difficulty,
            last_state: data,
        })
    }

    fn start_update(&mut self, _time: GameTime, data: Self::UpdateData) {
        self.last_state = data;
    }

    fn push_event(&mut self, time: GameTime, event: EventType<G>) {
        self.cur_segment.push_event(SegmentEvent::new(time, event));
    }

    fn end_update(&mut self, time: GameTime, new_location: Option<Location<G>>) {
        if let Some(location) = new_location {
            self.push_segment(StageSegment::new(location, time, self.last_state.clone()))
        }
    }

    fn finish(self, time: GameTime, cleared: bool) -> Self::Output {
        let Self {
            mut mode,
            start,
            shot,
            difficulty,
            last_state,
            mut cur_segment,
        } = self;

        if !cleared {
            cur_segment.set_success(false);
        }
        mode.push_segment(cur_segment);

        Run::new(
            shot,
            difficulty,
            start,
            StartEnd::new(time, last_state),
            match mode {
                TrackerMode::Full(stages) => RunType::Full { stages, cleared },
                TrackerMode::StagePractice(stage) => RunType::StagePractice(stage),
                TrackerMode::SpellPractice(last) => RunType::SpellPractice(last.unwrap()),
            },
        )
    }
}
