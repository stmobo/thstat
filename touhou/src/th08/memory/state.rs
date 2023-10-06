use std::io::{Error as IOError, ErrorKind, Result as IOResult};

use super::location::Location;
use super::process::MemoryAccess;
use crate::memory::traits::*;
use crate::memory::{
    define_state_struct, ensure_float_within_range, try_into_or_io_error, wrap_io_error, SpellState,
};
use crate::th08::{Difficulty, ShotType, SpellId, Stage, Touhou8};
use crate::types::Gen1Power;

define_state_struct! {
    PlayerState {
        character: ShotType,
        difficulty: Difficulty,
        lives: u8,
        bombs: u8,
        power: Gen1Power,
        continues: u8,
        total_misses: u32,
        total_bombs: u32,
        score: u32,
        gauge: u16,
        value: u32,
        night: u8,
        time: u32
    }
}

impl PlayerState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let character = proc
            .character()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let lives = ensure_float_within_range!(proc.player_lives()? => u8 : (0, 8, "lives"));
        let bombs = ensure_float_within_range!(proc.player_bombs()? => u8 : (0, 8, "bombs"));
        let power = ensure_float_within_range!(proc.player_power()? => u8 : (0, 128, "power"))
            .try_into()
            .map_err(wrap_io_error(ErrorKind::InvalidData))?;
        let continues = proc.continues_used()?;

        Ok(Self {
            character,
            difficulty,
            lives,
            bombs,
            power,
            continues,
            total_misses: proc.misses()? as u32,
            total_bombs: proc.bombs_used()? as u32,
            score: proc.score_1()?,
            gauge: proc.gauge()?,
            value: proc.value()?,
            night: proc.night()?,
            time: proc.time_1()?,
        })
    }
}

impl PlayerData<Touhou8> for PlayerState {
    fn shot(&self) -> ShotType {
        self.character
    }

    fn power(&self) -> Gen1Power {
        self.power
    }

    fn lives(&self) -> u8 {
        self.lives
    }

    fn continues_used(&self) -> u8 {
        self.continues
    }

    fn score(&self) -> u64 {
        self.score as u64
    }
}

impl BombStock<Touhou8> for PlayerState {
    fn bombs(&self) -> u8 {
        self.bombs
    }
}

impl MissCount<Touhou8> for PlayerState {
    fn total_misses(&self) -> u8 {
        self.total_misses as u8
    }
}

impl BombCount<Touhou8> for PlayerState {
    fn total_bombs(&self) -> u8 {
        self.total_bombs as u8
    }
}

define_state_struct! {
    BossState {
        remaining_lifebars: u32,
        active_spell: Option<SpellState<Touhou8>>,
    }
}

impl BossState {
    fn read_active_spell(proc: &MemoryAccess) -> IOResult<Option<SpellState<Touhou8>>> {
        let spell_status = proc.cur_spell_state()?;
        if (spell_status & 1) != 0 {
            let spell_id = proc.cur_spell_id()? + 1;
            let captured = (spell_status & 4) != 0;

            SpellId::try_from(spell_id)
                .map(|spell| Some(SpellState::new(spell, captured)))
                .map_err(move |error| IOError::new(ErrorKind::InvalidData, error))
        } else {
            Ok(None)
        }
    }

    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        Ok(Self {
            remaining_lifebars: proc.boss_healthbars()?,
            active_spell: Self::read_active_spell(proc)?,
        })
    }
}

impl BossData<Touhou8> for BossState {
    fn active_spell(&self) -> Option<SpellState<Touhou8>> {
        self.active_spell
    }
}

impl BossLifebars<Touhou8> for BossState {
    fn remaining_lifebars(&self) -> u8 {
        self.remaining_lifebars as u8
    }
}

define_state_struct! {
    StageState {
        stage: Stage,
        frame: u32,
        boss_state: Option<BossState>
    }
}

impl StageState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let stage = proc
            .stage()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let boss_state = if proc.boss_active()? != 0 {
            Some(BossState::new(proc)?)
        } else {
            None
        };

        Ok(Self {
            stage,
            boss_state,
            frame: proc.frame()?,
        })
    }
}

impl StageData<Touhou8> for StageState {
    type BossState = BossState;

    fn stage_id(&self) -> Stage {
        self.stage
    }

    fn active_boss(&self) -> Option<&Self::BossState> {
        self.boss_state.as_ref()
    }
}

impl ECLTimeline<Touhou8> for StageState {
    fn ecl_time(&self) -> u32 {
        self.frame()
    }
}

define_state_struct! {
    RunState {
        difficulty: Difficulty,
        player: PlayerState,
        stage: StageState,
        paused: bool,
    }
}

impl RunState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        Ok(Self {
            difficulty,
            player: PlayerState::new(proc)?,
            stage: StageState::new(proc)?,
            paused: (proc.game_mode()? & 0x04) == 0,
        })
    }
}

impl RunData<Touhou8> for RunState {
    type StageState = StageState;
    type PlayerState = PlayerState;

    fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    fn player(&self) -> &PlayerState {
        &self.player
    }

    fn stage(&self) -> &StageState {
        &self.stage
    }
}

impl PauseState for RunState {
    fn paused(&self) -> bool {
        self.paused
    }
}

impl ResolveLocation<Touhou8> for RunState {
    fn resolve_location(&self) -> Option<Location> {
        Location::resolve(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameType {
    Main(RunState),
    StagePractice(RunState),
    SpellPractice(PlayerState, SpellState<Touhou8>, bool),
}

impl PauseState for GameType {
    fn paused(&self) -> bool {
        match self {
            Self::Main(state) | Self::StagePractice(state) => state.paused,
            Self::SpellPractice(_, _, paused) => *paused,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    PlayerData,
    MusicRoom,
    GameStartMenu,
    PracticeStartMenu,
    UnknownMenu {
        menu_state: u32,
    },
    InGame {
        game: GameType,
        paused: bool,
    },
    InReplay {
        game: GameType,
        demo: bool,
        paused: bool,
    },
    ReplayEnded,
    GameOver {
        cleared: bool,
        game: GameType,
    },
    LoadingStage,
    RetryingGame,
    Unknown {
        state_id: u32,
        mode: u32,
    },
}

impl GameState {
    pub fn run_is_active(proc: &MemoryAccess) -> IOResult<bool> {
        let mode = proc.game_mode()?;
        let state = proc.program_state()?;
        let replay = (mode & 0x08) != 0;
        let spell_practice = ((mode & 0x0180) != 0) || ((mode & 0x4000) != 0);
        Ok((state == 2 || state == 3 || state == 10) && !replay && !spell_practice)
    }

    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let mode = proc.game_mode()?;
        let practice = (mode & 0x01) != 0;
        let demo = (mode & 0x02) != 0;
        let paused = (mode & 0x04) == 0; // bit is set if UNpaused
        let replay = (mode & 0x08) != 0;
        let cleared = (mode & 0x10) != 0;
        let spell_practice = ((mode & 0x0180) != 0) || ((mode & 0x4000) != 0);

        match proc.program_state()? {
            1 => Ok(match proc.menu_state()? {
                8 => GameState::MusicRoom,
                5 => GameState::PlayerData,
                1 => {
                    if practice {
                        GameState::PracticeStartMenu
                    } else {
                        GameState::GameStartMenu
                    }
                }
                other => GameState::UnknownMenu { menu_state: other },
            }),
            2 => {
                let game = if spell_practice {
                    if let Some(active_spell) = BossState::read_active_spell(proc)? {
                        GameType::SpellPractice(PlayerState::new(proc)?, active_spell, paused)
                    } else {
                        return Ok(GameState::Unknown { state_id: 2, mode });
                    }
                } else if practice {
                    GameType::StagePractice(RunState::new(proc)?)
                } else {
                    GameType::Main(RunState::new(proc)?)
                };

                if replay {
                    Ok(GameState::InReplay { game, demo, paused })
                } else {
                    Ok(GameState::InGame { game, paused })
                }
            }
            3 => Ok(GameState::LoadingStage),
            6 | 7 | 9 => {
                if replay {
                    Ok(GameState::ReplayEnded)
                } else {
                    let game = if spell_practice {
                        if let Some(active_spell) = BossState::read_active_spell(proc)? {
                            GameType::SpellPractice(PlayerState::new(proc)?, active_spell, paused)
                        } else {
                            return Ok(GameState::Unknown { state_id: 2, mode });
                        }
                    } else if practice {
                        GameType::StagePractice(RunState::new(proc)?)
                    } else {
                        GameType::Main(RunState::new(proc)?)
                    };

                    Ok(GameState::GameOver { cleared, game })
                }
            }
            10 => Ok(GameState::RetryingGame),
            state_id @ (5 | 8 | 11..=12) => Ok(GameState::Unknown { state_id, mode }), // used by the game, but unidentified for now
            0xFFFFFFFF => Err(IOError::new(ErrorKind::NotConnected, "game is not ready")), // set during startup and shutdown
            other @ (0 | 4 | 13..=0xFFFFFFFE) => Err(IOError::new(
                ErrorKind::InvalidData,
                format!("invalid game state value {}", other),
            )),
        }
    }
}
