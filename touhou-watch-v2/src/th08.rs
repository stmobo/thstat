use touhou::memory::ResolveLocation;
use touhou::th08::memory::{GameMemory, GameState, GameType, ReadResult, RunState};
use touhou::Touhou8;

use crate::set_track::{ActiveGame, Metrics, SetTracker};
use crate::watcher::{GameReader, TrackedGame};

impl TrackedGame for Touhou8 {
    type Reader = ReadWrapper;

    fn autodetect_process() -> ReadResult<Option<Self::Reader>> {
        GameMemory::new().map(|x| {
            x.map(|memory| ReadWrapper {
                memory,
                state: None,
            })
        })
    }

    fn get_tracker(metrics: &Metrics) -> &SetTracker<Self> {
        metrics.th08()
    }

    fn get_tracker_mut(metrics: &mut Metrics) -> &mut SetTracker<Self> {
        metrics.th08_mut()
    }
}

#[derive(Debug)]
struct State {
    tracking: ActiveGame<Touhou8>,
    misses: u32,
    bombs: u8,
    total_bombs: u32,
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
            bombs: player.bombs(),
            total_bombs: player.total_bombs(),
        }
    }

    fn update(&mut self, memory_state: &RunState) -> bool {
        let player = memory_state.player();
        let stage = memory_state.stage();
        let mut interesting = false;

        // if self.tracking.update_spell(&stage) {
        //     interesting = true;
        // }

        if let Some(location) = memory_state.resolve_location() {
            interesting = self.tracking.update_location(location);
        }

        if player.total_misses() > self.misses
            || player.bombs() < self.bombs
            || player.total_bombs() > self.total_bombs
        {
            self.tracking.mark_failed();
            interesting = true;
        }

        self.tracking.update_pause(memory_state);
        self.misses = player.total_misses();
        self.bombs = player.bombs();
        self.total_bombs = player.total_bombs();

        interesting
    }

    fn end_update(mut self, memory_state: &RunState, success: bool) {
        self.update(memory_state);
        self.tracking.mark_cleared(success);
    }
}

#[derive(Debug)]
pub struct ReadWrapper {
    memory: GameMemory,
    state: Option<State>,
}

impl GameReader<Touhou8> for ReadWrapper {
    fn is_in_game(&mut self) -> ReadResult<Option<bool>> {
        self.memory
            .access()
            .map(GameState::run_is_active)
            .transpose()
    }

    fn reset(&mut self) {
        match self.memory.access().map(GameState::new).transpose() {
            Ok(Some(GameState::GameOver { cleared, game })) => match game {
                GameType::Main(run) => {
                    if let Some(state) = self.state.take() {
                        state.end_update(&run, cleared);
                    }
                }
                GameType::StagePractice(run) => {
                    if let Some(state) = self.state.take() {
                        state.end_update(&run, true);
                    }
                }
                GameType::SpellPractice(_, _, _) => {
                    self.state = None;
                }
            },
            _ => {
                self.state = None;
            }
        }
    }

    fn update(&mut self) -> ReadResult<bool> {
        match self.memory.access().map(GameState::new).transpose()? {
            Some(GameState::InGame { game, .. }) => match game {
                GameType::Main(run) | GameType::StagePractice(run) => {
                    if let Some(state) = &mut self.state {
                        Ok(state.update(&run))
                    } else {
                        self.state = Some(State::new(&run));
                        Ok(true)
                    }
                }
                GameType::SpellPractice(_, _, _) => Ok(self.state.take().is_some()),
            },
            Some(GameState::GameOver { cleared, game }) => match game {
                GameType::Main(run) => {
                    if let Some(state) = self.state.take() {
                        state.end_update(&run, cleared);
                    }
                    Ok(true)
                }
                GameType::StagePractice(run) => {
                    if let Some(state) = self.state.take() {
                        state.end_update(&run, true);
                    }
                    Ok(true)
                }
                GameType::SpellPractice(_, _, _) => Ok(self.state.take().is_some()),
            },
            Some(GameState::LoadingStage) => Ok(false),
            _ => Ok(self.state.take().is_some()),
        }
    }

    fn pid(&self) -> u32 {
        self.memory.pid()
    }
}
