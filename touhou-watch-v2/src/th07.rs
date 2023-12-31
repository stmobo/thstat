use std::time::{Duration, Instant};

use touhou::memory::ResolveLocation;
use touhou::th07::memory::{GameMemory, GameState, ReadResult, RunState};
use touhou::Touhou7;

use crate::set_track::{ActiveGame, Metrics, SetTracker};
use crate::watcher::{GameReader, TrackedGame};

#[derive(Debug, Clone)]
struct State {
    tracking: ActiveGame<Touhou7>,
    misses: u32,
    bombs: u32,
    border_start: Option<Instant>,
}

impl State {
    fn new(memory_state: &RunState) -> Self {
        let player = memory_state.player();
        let mut tracking = ActiveGame::new(memory_state);

        if let Some(location) = memory_state.resolve_location() {
            tracking.update_location(location);
        }

        tracking.update_pause(memory_state);

        Self {
            tracking,
            misses: player.total_misses(),
            bombs: player.total_bombs(),
            border_start: if player.border_active() {
                Some(Instant::now())
            } else {
                None
            },
        }
    }

    fn update(&mut self, memory_state: &RunState) -> bool {
        let player = memory_state.player();
        let stage = memory_state.stage();
        let mut interesting = false;

        if self.tracking.update_spell(&stage) {
            interesting = true;
        }

        if let Some(location) = memory_state.resolve_location() {
            interesting = self.tracking.update_location(location);
        }

        if player.total_misses() > self.misses || player.total_bombs() > self.bombs {
            self.tracking.mark_failed();
            interesting = true;
        }

        match (self.border_start, player.border_active()) {
            (None, true) => self.border_start = Some(Instant::now()),
            (Some(border_start), false) => {
                let duration = Instant::now().duration_since(border_start);

                if duration < Duration::from_millis(8750)
                    && !self
                        .tracking
                        .cur_location()
                        .is_some_and(|location| location.is_boss_start())
                {
                    self.tracking.mark_failed();
                    interesting = true;
                }

                self.border_start = None;
            }
            _ => {}
        }

        self.tracking.update_pause(memory_state);
        self.misses = player.total_misses();
        self.bombs = player.total_bombs();

        interesting
    }

    fn end_update(mut self, memory_state: &RunState, success: bool) {
        self.update(memory_state);
        self.tracking.mark_cleared(success);
    }
}

#[derive(Debug)]
pub struct MemoryWrapper {
    memory: GameMemory,
    state: Option<State>,
}

impl TrackedGame for Touhou7 {
    type Reader = MemoryWrapper;

    fn autodetect_process() -> ReadResult<Option<Self::Reader>> {
        GameMemory::new().map(|x| {
            x.map(|memory| MemoryWrapper {
                memory,
                state: None,
            })
        })
    }

    fn get_tracker(metrics: &Metrics) -> &SetTracker<Self> {
        metrics.th07()
    }

    fn get_tracker_mut(metrics: &mut Metrics) -> &mut SetTracker<Self> {
        metrics.th07_mut()
    }
}

impl GameReader<Touhou7> for MemoryWrapper {
    fn is_in_game(&mut self) -> ReadResult<Option<bool>> {
        self.memory
            .access()
            .map(GameState::game_is_active)
            .transpose()
    }

    fn reset(&mut self) {
        self.state = None;
    }

    fn update(&mut self) -> ReadResult<bool> {
        match dbg!(self.memory.access().map(GameState::new).transpose()?) {
            Some(GameState::InGame { run, .. }) => {
                if let Some(state) = &mut self.state {
                    Ok(state.update(&run))
                } else {
                    self.state = Some(State::new(&run));
                    Ok(true)
                }
            }
            Some(GameState::GameOver { cleared, run }) => {
                if let Some(state) = self.state.take() {
                    state.end_update(&run, cleared);
                }

                Ok(true)
            }
            Some(GameState::LoadingStage) => Ok(false),
            _ => Ok(self.state.take().is_some()),
        }
    }

    fn pid(&self) -> u32 {
        self.memory.pid()
    }
}
