use std::io::{Error as IOError, ErrorKind, Result as IOResult};

use serde::{Deserialize, Serialize};

use super::process::MemoryAccess;
use crate::memory::{define_state_struct, ensure_float_within_range, try_into_or_io_error};
use crate::th08::{Difficulty, Stage, Touhou8};
use crate::types::ShotType;

define_state_struct! {
    PlayerState {
        character: ShotType<Touhou8>,
        difficulty: Difficulty,
        lives: u8,
        bombs: u8,
        power: u8,
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
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))
            .map(ShotType::new)?;

        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let lives = ensure_float_within_range!(proc.player_lives()? => u8 : (0, 8, "lives"));
        let bombs = ensure_float_within_range!(proc.player_bombs()? => u8 : (0, 8, "bombs"));
        let power = ensure_float_within_range!(proc.player_power()? => u8 : (0, 128, "power"));
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

define_state_struct! {
    ActiveSpell {
        id: u32,
        captured: bool
    }
}

impl ActiveSpell {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        Ok(Self {
            id: proc.cur_spell_id()?,
            captured: (proc.cur_spell_state()? & 4) != 0,
        })
    }
}

define_state_struct! {
    BossState {
        remaining_lifebars: u32,
        active_spell: Option<ActiveSpell>,
    }
}

impl BossState {
    pub fn new(proc: &MemoryAccess) -> IOResult<Self> {
        let spell_status = proc.cur_spell_state()?;

        let active_spell = if (spell_status & 1) != 0 {
            Some(ActiveSpell::new(proc)?)
        } else {
            None
        };

        Ok(Self {
            remaining_lifebars: proc.boss_healthbars()?,
            active_spell,
        })
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
            .and_then(|v| {
                v.checked_sub(1).ok_or(IOError::new(
                    ErrorKind::InvalidData,
                    "invalid value 0 for stage",
                ))
            })
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

#[derive(Debug, Clone, Copy)]
pub enum GameType {
    Main(StageState),
    StagePractice(StageState),
    SpellPractice(ActiveSpell),
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
        player: PlayerState,
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
        player: PlayerState,
    },
    LoadingStage,
    RetryingGame,
    Unknown {
        state_id: u32,
        mode: u32,
    },
}

impl GameState {
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
                    GameType::SpellPractice(ActiveSpell::new(proc)?)
                } else if practice {
                    GameType::StagePractice(StageState::new(proc)?)
                } else {
                    GameType::Main(StageState::new(proc)?)
                };

                if replay {
                    Ok(GameState::InReplay { game, demo, paused })
                } else {
                    Ok(GameState::InGame {
                        game,
                        paused,
                        player: PlayerState::new(proc)?,
                    })
                }
            }
            3 => Ok(GameState::LoadingStage),
            6 | 7 | 9 => {
                if replay {
                    Ok(GameState::ReplayEnded)
                } else {
                    let game = if spell_practice {
                        GameType::SpellPractice(ActiveSpell::new(proc)?)
                    } else if practice {
                        GameType::StagePractice(StageState::new(proc)?)
                    } else {
                        GameType::Main(StageState::new(proc)?)
                    };

                    Ok(GameState::GameOver {
                        cleared,
                        game,
                        player: PlayerState::new(proc)?,
                    })
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
