use touhou::memory::ResolveLocation;
use touhou::th10::memory::{GameMemory, GameState, RunState};
use touhou::th10::ReadResult;
use touhou::{ShotPower, Touhou10};

use crate::set_track::{ActiveGame, Metrics, SetTracker};
use crate::watcher::{GameReader, TrackedGame};

#[derive(Debug)]
struct State {
    lives: u8,
    power: ShotPower<Touhou10>,
    tracking: ActiveGame<Touhou10>,
}

impl State {
    fn new(memory_state: &RunState) -> Self {
        let player = memory_state.player();
        let mut tracking = ActiveGame::new(memory_state);

        if let Some(location) = memory_state.resolve_location() {
            if !location.is_stage_section() {
                tracking.update_location(location);
            }
        }

        Self {
            lives: player.lives(),
            power: player.power(),
            tracking,
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
            if !location.is_stage_section() {
                interesting = self.tracking.update_location(location);
            } else {
                self.tracking.exit_location();
            };
        }

        if player.lives() < self.lives || player.power() < self.power {
            self.tracking.mark_failed();
            interesting = true;
        }

        self.lives = player.lives();
        self.power = player.power();

        interesting
    }

    fn end_update(mut self, memory_state: &RunState, success: bool) {
        self.update(memory_state);
        self.tracking.mark_cleared(success);
    }
}

#[derive(Debug)]
pub struct ReadWrapper {
    reader: GameMemory,
    state: Option<State>,
}

impl TrackedGame for Touhou10 {
    type Reader = ReadWrapper;

    fn autodetect_process() -> ReadResult<Option<Self::Reader>> {
        GameMemory::new().map(|x| {
            x.map(|reader| ReadWrapper {
                reader,
                state: None,
            })
        })
    }

    fn get_tracker(metrics: &Metrics) -> &SetTracker<Self> {
        metrics.th10()
    }

    fn get_tracker_mut(metrics: &mut Metrics) -> &mut SetTracker<Self> {
        metrics.th10_mut()
    }
}

impl GameReader<Touhou10> for ReadWrapper {
    fn is_in_game(&mut self) -> ReadResult<Option<bool>> {
        self.reader.access().map(GameState::is_in_game).transpose()
    }

    fn reset(&mut self) {
        self.state = None;
    }

    fn update(&mut self) -> ReadResult<bool> {
        match self.reader.access().map(GameState::new).transpose()? {
            Some(GameState::InGame(run)) => {
                if let Some(state) = &mut self.state {
                    Ok(state.update(&run))
                } else {
                    self.state = Some(State::new(&run));
                    Ok(true)
                }
            }
            Some(GameState::Ending(run)) => {
                if let Some(state) = self.state.take() {
                    state.end_update(&run, true);
                }

                Ok(true)
            }
            Some(_) => Ok(self.state.take().is_some()),
            None => Ok(false),
        }
    }

    fn pid(&self) -> u32 {
        self.reader.pid()
    }
}
