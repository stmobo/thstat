use std::fmt::Display;
use std::io::{Error as IOError, ErrorKind, Result as IOResult};

use serde::{Deserialize, Serialize};

use super::location::Location;
use super::process::GameMemory;
use crate::memory::traits::*;
use crate::memory::{
    define_state_struct, ensure_float_within_range, try_into_or_io_error, SpellState,
};
use crate::th07::{Difficulty, ShotType, SpellId, Stage, Touhou7};
use crate::SpellCard;

define_state_struct! {
    PlayerState {
        character: ShotType,
        lives: u8,
        bombs: u8,
        power: u8,
        continues: u8,
        total_misses: u32,
        total_bombs: u32,
        border_active: bool,
        score: u32,
        cherry: u32,
        cherry_max: u32,
        cherry_plus: u32
    }
}

impl PlayerState {
    pub fn new(proc: &GameMemory) -> IOResult<Self> {
        let character = proc
            .player_character()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let lives = ensure_float_within_range!(proc.player_lives()? => u8 : (0, 8, "lives"));
        let bombs = ensure_float_within_range!(proc.player_bombs()? => u8 : (0, 8, "bombs"));
        let power = ensure_float_within_range!(proc.player_power()? => u8 : (0, 128, "power"));
        let continues = proc.player_continues()?;

        if !(0..=5).contains(&continues) {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!("invalid continue count (got {}, expected 0..=5)", continues),
            ));
        }

        let cherry_base = proc.cherry_base()?;
        let cherry_max = proc.cherry_max()?.saturating_sub(cherry_base);

        Ok(Self {
            character,
            lives,
            bombs,
            power,
            continues,
            total_misses: proc.player_misses()? as u32,
            total_bombs: proc.player_bombs_used()? as u32,
            border_active: proc.border_state()? != 0,
            score: proc.score()?,
            cherry_max,
            cherry: proc.cherry()?.saturating_sub(cherry_base).min(cherry_max),
            cherry_plus: proc.cherry_plus()?.saturating_sub(cherry_base).min(50000),
        })
    }
}

impl PlayerData<Touhou7> for PlayerState {
    fn shot(&self) -> ShotType {
        self.character
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

impl BombStock for PlayerState {
    fn bombs(&self) -> u8 {
        self.bombs
    }
}

impl MissCount for PlayerState {
    fn total_misses(&self) -> u8 {
        self.total_misses as u8
    }
}

impl BombCount for PlayerState {
    fn total_bombs(&self) -> u8 {
        self.total_bombs as u8
    }
}

define_state_struct! {
    BossState {
        id: u8,
        is_midboss: bool,
        remaining_lifebars: u32,
        active_spell: Option<SpellState<Touhou7>>,
    }
}

impl BossState {
    pub fn new(proc: &GameMemory) -> IOResult<Self> {
        let active_spell = if proc.spell_active()? != 0 {
            let spell_id = proc.current_spell_id()? + 1;
            let captured = proc.spell_captured()? != 0;
            SpellId::try_from(spell_id)
                .ok()
                .map(|spell| SpellState::new(spell, captured))
        } else {
            None
        };

        Ok(Self {
            id: proc.boss_id()?,
            is_midboss: proc.midboss_flag()? != 3,
            remaining_lifebars: proc.boss_healthbars()?,
            active_spell,
        })
    }
}

impl BossData<Touhou7> for BossState {
    fn active_spell(&self) -> Option<SpellState<Touhou7>> {
        self.active_spell
    }
}

impl BossLifebars for BossState {
    fn remaining_lifebars(&self) -> u8 {
        self.remaining_lifebars as u8
    }
}

define_state_struct! {
    StageState {
        stage: Stage,
        ecl_time: u32,
        boss_state: Option<BossState>
    }
}

impl StageState {
    pub fn new(proc: &GameMemory) -> IOResult<Self> {
        let stage = proc
            .stage()
            .and_then(|v| {
                v.checked_sub(1).ok_or(IOError::new(
                    ErrorKind::InvalidData,
                    "invalid value 0 for stage",
                ))
            })
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let boss_state = if proc.boss_flag()? != 0 {
            Some(BossState::new(proc)?)
        } else {
            None
        };

        Ok(Self {
            stage,
            boss_state,
            ecl_time: proc.ecl_time()?,
        })
    }
}

impl StageData<Touhou7> for StageState {
    type BossState = BossState;

    fn stage_id(&self) -> Stage {
        self.stage
    }

    fn ecl_time(&self) -> u32 {
        self.ecl_time
    }

    fn active_boss(&self) -> Option<&Self::BossState> {
        self.boss_state.as_ref()
    }
}

define_state_struct! {
    RunState {
        difficulty: Difficulty,
        player: PlayerState,
        stage: StageState
    }
}

impl RunState {
    pub fn new(proc: &GameMemory) -> IOResult<Self> {
        let difficulty = proc
            .difficulty()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        Ok(Self {
            difficulty,
            player: PlayerState::new(proc)?,
            stage: StageState::new(proc)?,
        })
    }

    pub fn location(&self) -> Option<Location> {
        Location::resolve(self)
    }
}

impl RunData<Touhou7> for RunState {
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

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    TitleScreen,
    PlayerData,
    MusicRoom,
    GameStartMenu,
    PracticeStartMenu,
    UnknownMenu {
        menu_state: u32,
    },
    InGame {
        practice: bool,
        paused: bool,
        run: RunState,
    },
    InReplay {
        practice: bool,
        demo: bool,
        paused: bool,
    },
    ReplayEnded,
    GameOver {
        cleared: bool,
        run: RunState,
    },
    LoadingStage,
    RetryingGame,
    Unknown {
        state_id: u32,
        mode: u8,
    },
}

impl GameState {
    pub fn new(proc: &GameMemory) -> IOResult<Self> {
        let mode = proc.game_mode()?;
        let practice = (mode & 0x01) != 0;
        let demo = (mode & 0x02) != 0;
        let paused = (mode & 0x04) == 0; // bit is set if UNpaused
        let replay = (mode & 0x08) != 0;
        let cleared = (mode & 0x10) != 0;

        /* the program sets the game state value to values from 1 through 3 and 5 through 12 (inclusive) during regular operation,
         * and to 0xFFFFFFFF when starting up and shutting down.
         *
         */

        match proc.game_state()? {
            1 => Ok(match proc.menu_state()? {
                35 => GameState::MusicRoom,
                47 => GameState::PlayerData,
                129 => {
                    if practice {
                        GameState::PracticeStartMenu
                    } else {
                        GameState::GameStartMenu
                    }
                }
                130 => GameState::TitleScreen,
                other => GameState::UnknownMenu { menu_state: other },
            }),
            2 => {
                if replay {
                    Ok(GameState::InReplay {
                        practice,
                        demo,
                        paused,
                    })
                } else {
                    Ok(GameState::InGame {
                        practice,
                        paused,
                        run: RunState::new(proc)?,
                    })
                }
            }
            3 => Ok(GameState::LoadingStage),
            6 | 7 | 9 => {
                if replay {
                    Ok(GameState::ReplayEnded)
                } else {
                    Ok(GameState::GameOver {
                        cleared,
                        run: RunState::new(proc)?,
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

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::TitleScreen => f.write_str("At Title Screen"),
            Self::PlayerData => f.write_str("Viewing Player Data"),
            Self::MusicRoom => f.write_str("In Music Room"),
            Self::GameStartMenu => f.write_str("In Game Start Menu"),
            Self::PracticeStartMenu => f.write_str("In Practice Start Menu"),
            Self::UnknownMenu { menu_state } => write!(f, "In unknown menu {}", menu_state),
            Self::InGame {
                practice, paused, ..
            } => {
                if practice {
                    f.write_str("In Practice Game")?;
                } else {
                    f.write_str("In Game")?;
                }

                if paused {
                    f.write_str(" (Paused)")
                } else {
                    Ok(())
                }
            }
            Self::InReplay {
                practice,
                demo,
                paused,
            } => {
                if demo {
                    f.write_str("Viewing Demo")?;
                } else if practice {
                    f.write_str("Viewing Practice Replay")?;
                } else {
                    f.write_str("Viewing Replay")?;
                }

                if paused {
                    f.write_str(" (Paused)")
                } else {
                    Ok(())
                }
            }
            Self::ReplayEnded => f.write_str("At End of Replay"),
            Self::GameOver { cleared, .. } => {
                if cleared {
                    f.write_str("Cleared Game")
                } else {
                    f.write_str("Game Over")
                }
            }
            Self::LoadingStage => f.write_str("Loading Next Stage"),
            Self::RetryingGame => f.write_str("Loading Retry"),
            Self::Unknown { state_id, mode } => {
                write!(f, "Unknown state {} (mode {})", state_id, mode)
            }
        }
    }
}
