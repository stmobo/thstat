#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::VecDeque;
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::{ErrorKind, Result as IOResult};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{env, fs, io};

use process_memory::{
    Architecture, DataMember, Memory, ProcessHandle, ProcessHandleExt, TryIntoProcessHandle,
};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};
use tauri::{Manager, Window};
use time::OffsetDateTime;
use touhou::{Difficulty, ShotType, SpellCard, Stage, Touhou7};

#[derive(Debug, Clone)]
pub struct Touhou7Memory {
    pid: u32,
    stage: DataMember<u32>,
    menu_state: DataMember<u32>,
    game_state: DataMember<u32>,
    game_mode: DataMember<u8>,
    difficulty: DataMember<u32>,
    ecl_time: DataMember<u32>,
    spell_captured: DataMember<u32>,
    spell_active: DataMember<u32>,
    current_spell_id: DataMember<u32>,
    boss_flag: DataMember<u32>,
    midboss_flag: DataMember<u8>,
    boss_healthbars: DataMember<u32>,
    boss_id: DataMember<u8>,
    player_character: DataMember<u8>,
    player_lives: DataMember<f32>,
    player_bombs: DataMember<f32>,
    player_power: DataMember<f32>,
    player_misses: DataMember<f32>,
    player_bombs_used: DataMember<f32>,
    player_continues: DataMember<u8>,
    border_state: DataMember<u8>,
}

impl Touhou7Memory {
    fn find_process(system: &System) -> Option<&Process> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find(|&process| {
                process
                    .exe()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("th07"))
                    .unwrap_or(false)
            })
            .filter(|proc| proc.run_time() > 15)
    }

    pub fn new_autodetect(system: &System) -> IOResult<Option<Self>> {
        if let Some(proc) = Self::find_process(system) {
            Self::new(proc.pid().as_u32()).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn new(pid: u32) -> IOResult<Self> {
        let handle = pid
            .try_into_process_handle()?
            .set_arch(Architecture::Arch32Bit);

        Ok(Self {
            pid,
            stage: DataMember::new_offset(handle, vec![0x0062f85c]),
            menu_state: DataMember::new_offset(handle, vec![0x004b9e44, 0x0c]),
            game_state: DataMember::new_offset(handle, vec![0x00575aa8]),
            game_mode: DataMember::new_offset(handle, vec![0x0062f648]),
            difficulty: DataMember::new_offset(handle, vec![0x00626280]),
            ecl_time: DataMember::new_offset(handle, vec![0x009a9af8, 0x009545fc]),
            spell_active: DataMember::new_offset(handle, vec![0x012fe0c8]),
            spell_captured: DataMember::new_offset(handle, vec![0x012fe0c4]),
            current_spell_id: DataMember::new_offset(handle, vec![0x012fe0d8]),
            boss_flag: DataMember::new_offset(handle, vec![0x0049fc14]),
            midboss_flag: DataMember::new_offset(handle, vec![0x009b655a]),
            boss_id: DataMember::new_offset(handle, vec![0x009b1879]),
            boss_healthbars: DataMember::new_offset(handle, vec![0x0049fc08]),
            player_character: DataMember::new_offset(handle, vec![0x0062f647]),
            player_lives: DataMember::new_offset(handle, vec![0x00626278, 0x5c]),
            player_bombs: DataMember::new_offset(handle, vec![0x00626278, 0x68]),
            player_power: DataMember::new_offset(handle, vec![0x00626278, 0x7c]),
            player_misses: DataMember::new_offset(handle, vec![0x00626278, 0x50]),
            player_bombs_used: DataMember::new_offset(handle, vec![0x00626278, 0x6c]),
            player_continues: DataMember::new_offset(handle, vec![0x00626278, 0x20]),
            border_state: DataMember::new_offset(handle, vec![0x004bdad8 + 0x240d]),
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_running(&self, system: &mut System) -> bool {
        system.refresh_process_specifics(Pid::from_u32(self.pid), ProcessRefreshKind::new())
    }

    pub fn get_game_state(&self) -> IOResult<GameState> {
        unsafe {
            let mode = self.game_mode.read()?;
            let practice = (mode & 0x01) != 0;
            let demo = (mode & 0x02) != 0;
            let paused = (mode & 0x04) == 0; // bit is set if UNpaused
            let replay = (mode & 0x08) != 0;
            let cleared = (mode & 0x10) != 0;

            /* the program sets the game state value to values from 1 through 3 and 5 through 12 (inclusive) during regular operation,
             * and to 0xFFFFFFFF when starting up and shutting down.
             *
             */

            match self.game_state.read()? {
                1 => Ok(match self.menu_state.read()? {
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
                            stage: self.get_stage_state()?,
                            player: self.get_player_state()?,
                        })
                    }
                }
                3 => Ok(GameState::LoadingStage),
                6 | 7 | 9 => {
                    if replay {
                        Ok(GameState::ReplayEnded)
                    } else {
                        Ok(GameState::GameOver { cleared })
                    }
                }
                10 => Ok(GameState::RetryingGame),
                state @ (5 | 8 | 11..=12) => Ok(GameState::Unknown { state, mode }), // used by the game, but unidentified for now
                0xFFFFFFFF => Err(io::Error::new(ErrorKind::NotConnected, "game is not ready")), // set during startup and shutdown
                other @ (0 | 4 | 13..=0xFFFFFFFE) => Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("invalid game state value {}", other),
                )),
            }
        }
    }

    pub fn get_player_state(&self) -> IOResult<PlayerState> {
        unsafe {
            let character = self
                .player_character
                .read()?
                .try_into()
                .map(ShotType::new)
                .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

            Ok(PlayerState {
                character,
                lives: self.player_lives.read()? as u8,
                bombs: self.player_bombs.read()? as u8,
                power: self.player_power.read()? as u8,
                continues: self.player_continues.read()?,
                total_misses: self.player_misses.read()? as u32,
                total_bombs: self.player_bombs_used.read()? as u32,
                border_active: self.border_state.read()? != 0,
            })
        }
    }

    pub fn get_stage_state(&self) -> IOResult<StageState> {
        unsafe {
            let boss_state = if self.boss_flag.read()? == 1 {
                let active_spell = if self.spell_active.read()? != 0 {
                    Some((
                        self.current_spell_id.read()?,
                        self.spell_captured.read()? != 0,
                    ))
                } else {
                    None
                };

                Some(BossState {
                    id: self.boss_id.read()?,
                    is_midboss: self.midboss_flag.read()? != 3,
                    remaining_lifebars: self.boss_healthbars.read()?,
                    active_spell,
                })
            } else {
                None
            };

            let difficulty = self.difficulty.read()? as u8;

            Ok(StageState {
                stage: self.stage.read()?,
                difficulty: difficulty
                    .try_into()
                    .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?,
                ecl_time: self.ecl_time.read()?,
                boss_state,
            })
        }
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
        stage: StageState,
        player: PlayerState,
    },
    InReplay {
        practice: bool,
        demo: bool,
        paused: bool,
    },
    ReplayEnded,
    GameOver {
        cleared: bool,
    },
    LoadingStage,
    RetryingGame,
    Unknown {
        state: u32,
        mode: u8,
    },
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
            Self::GameOver { cleared } => {
                if cleared {
                    f.write_str("Cleared Game")
                } else {
                    f.write_str("Game Over")
                }
            }
            Self::LoadingStage => f.write_str("Loading Next Stage"),
            Self::RetryingGame => f.write_str("Loading Retry"),
            Self::Unknown { state, mode } => write!(f, "Unknown state {} (mode {})", state, mode),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PlayerState {
    character: ShotType<Touhou7>,
    lives: u8,
    bombs: u8,
    power: u8,
    continues: u8,
    total_misses: u32,
    total_bombs: u32,
    border_active: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BossState {
    id: u8,
    is_midboss: bool,
    remaining_lifebars: u32,
    active_spell: Option<(u32, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StageSection {
    Start,
    FirstHalf { seq: u32 },
    MidbossNonspell { seq: u32 },
    MidbossSpell { seq: u32, spell: SpellCard<Touhou7> }, // sequence number, ID
    SecondHalf { seq: u32 },
    PreBoss,
    BossNonspell { seq: u32 },
    BossSpell { seq: u32, spell: SpellCard<Touhou7> },
    Unknown,
}

impl Display for StageSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start => f.write_str("Start"),
            Self::FirstHalf { seq } => write!(f, "First Half {}", *seq + 1),
            Self::MidbossNonspell { seq } => write!(f, "Midboss Nonspell {}", *seq + 1),
            Self::MidbossSpell { seq, spell } => {
                write!(
                    f,
                    "Midboss Spell {} (#{:03} {})",
                    *seq + 1,
                    spell.id(),
                    spell.name()
                )
            }
            Self::SecondHalf { seq } => write!(f, "Second Half {}", *seq + 1),
            Self::PreBoss => f.write_str("Pre-Boss"),
            Self::BossNonspell { seq } => write!(f, "Boss Nonspell {}", *seq + 1),
            Self::BossSpell { seq, spell } => write!(
                f,
                "Boss Spell {} (#{:03} {})",
                *seq + 1,
                spell.id(),
                spell.name()
            ),
            Self::Unknown => f.write_str("Unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StageLocation {
    stage: Stage,
    section: StageSection,
}

impl StageLocation {
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn section(&self) -> StageSection {
        self.section
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self.section(), StageSection::Unknown)
    }

    pub fn is_end_spell(&self) -> bool {
        matches!(
            (self.stage, self.section),
            (Stage::One, StageSection::BossSpell { seq: 1, .. })
                | (Stage::Two, StageSection::BossSpell { seq: 2, .. })
                | (
                    Stage::Three | Stage::Four | Stage::Five,
                    StageSection::BossSpell { seq: 3, .. }
                )
                | (Stage::Six, StageSection::BossSpell { seq: 5, .. })
        )
    }
}

impl Display for StageLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stage.fmt(f)?;

        if self.stage == Stage::Six && self.section == (StageSection::FirstHalf { seq: 1 }) {
            f.write_str(" Spam")
        } else if !self.is_unknown() {
            f.write_char(' ')?;
            self.section.fmt(f)
        } else {
            Ok(())
        }
    }
}

macro_rules! convert_to_spellcard {
    ($x:expr) => {
        SpellCard::new(($x + 1).try_into().unwrap())
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StageState {
    stage: u32,
    difficulty: Difficulty,
    ecl_time: u32,
    boss_state: Option<BossState>,
}

impl StageState {
    pub fn resolve_stage_section(&self) -> StageLocation {
        match self.stage {
            1 => StageLocation {
                stage: Stage::One,
                section: match self.ecl_time {
                    0..=539 => StageSection::Start,
                    540..=1340 => StageSection::FirstHalf { seq: 0 },
                    1341..=2655 => StageSection::FirstHalf { seq: 1 },
                    2656..=3106 => {
                        if let Some(boss) = &self.boss_state {
                            if let Some(spell) = &boss.active_spell {
                                StageSection::MidbossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(spell.0),
                                }
                            } else {
                                StageSection::MidbossNonspell { seq: 0 }
                            }
                        } else {
                            StageSection::SecondHalf { seq: 0 }
                        }
                    }
                    3107..=5041 => StageSection::SecondHalf { seq: 0 },
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 2..=5) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 6..=9) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    1 => StageSection::BossNonspell { seq: 0 },
                                    0 => StageSection::BossNonspell { seq: 1 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            2 => StageLocation {
                stage: Stage::Two,
                section: match self.ecl_time {
                    0..=389 => StageSection::Start,
                    390..=2825 => StageSection::FirstHalf { seq: 0 },
                    2826..=3365 => {
                        if let Some(boss) = &self.boss_state {
                            if let Some(spell) = &boss.active_spell {
                                StageSection::MidbossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(spell.0),
                                }
                            } else {
                                StageSection::MidbossNonspell { seq: 0 }
                            }
                        } else {
                            StageSection::SecondHalf { seq: 0 }
                        }
                    }
                    3366..=7646 => StageSection::SecondHalf { seq: 0 },
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 14..=17) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 18..=21) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 22..=25) => StageSection::BossSpell {
                                    seq: 2,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    1 => StageSection::BossNonspell { seq: 0 },
                                    0 => StageSection::BossNonspell { seq: 1 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            3 => StageLocation {
                stage: Stage::Three,
                section: match self.ecl_time {
                    0..=389 => StageSection::Start,
                    390..=820 => StageSection::FirstHalf { seq: 0 },
                    821..=1804 => {
                        if self.boss_state.is_some() {
                            StageSection::MidbossNonspell { seq: 0 }
                        } else {
                            StageSection::FirstHalf { seq: 1 }
                        }
                    }
                    1805..=1857 => StageSection::SecondHalf { seq: 0 },
                    1858..=3392 => {
                        if let Some(boss) = &self.boss_state {
                            if let Some(spell) = &boss.active_spell {
                                StageSection::MidbossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(spell.0),
                                }
                            } else {
                                StageSection::MidbossNonspell { seq: 1 }
                            }
                        } else {
                            StageSection::SecondHalf { seq: 0 }
                        }
                    }
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 28..=31) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 32..=35) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 36..=39) => StageSection::BossSpell {
                                    seq: 2,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 40..=43) => StageSection::BossSpell {
                                    seq: 3,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    2 => StageSection::BossNonspell { seq: 0 },
                                    1 => StageSection::BossNonspell { seq: 1 },
                                    0 => StageSection::BossNonspell { seq: 2 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            4 => StageLocation {
                stage: Stage::Four,
                section: match self.ecl_time {
                    0..=79 => StageSection::Start,
                    80..=1947 => StageSection::FirstHalf { seq: 0 },
                    1948..=3026 => StageSection::FirstHalf { seq: 1 },
                    3028..=4286 => StageSection::FirstHalf { seq: 2 },
                    4288..=7121 => StageSection::FirstHalf { seq: 3 },
                    7122..=7963 => StageSection::MidbossNonspell { seq: 0 },
                    7964..=10135 => {
                        if self.boss_state.is_some() {
                            StageSection::MidbossNonspell { seq: 0 }
                        } else {
                            StageSection::SecondHalf { seq: 0 }
                        }
                    }
                    10136..=11395 => StageSection::SecondHalf { seq: 1 },
                    11396..=13165 => StageSection::SecondHalf { seq: 2 },
                    13166..=14825 => StageSection::SecondHalf { seq: 3 },
                    14826..=15199 => StageSection::PreBoss,
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 44..=47) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 48..=51) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 52..=55) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 56..=59) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 60..=63) => StageSection::BossSpell {
                                    seq: 2,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 65..=68) => StageSection::BossSpell {
                                    seq: 3,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    3 => StageSection::BossNonspell { seq: 0 },
                                    2 => StageSection::BossNonspell { seq: 1 },
                                    1 => StageSection::BossNonspell { seq: 2 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            5 => StageLocation {
                stage: Stage::Five,
                section: match self.ecl_time {
                    0..=439 => StageSection::Start,
                    440..=839 => StageSection::FirstHalf { seq: 0 },
                    840..=2549 => StageSection::FirstHalf { seq: 1 },
                    2550..=4819 => StageSection::FirstHalf { seq: 2 },
                    4820..=4882 => {
                        if let Some(boss) = &self.boss_state {
                            if let Some(spell) = &boss.active_spell {
                                StageSection::MidbossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(spell.0),
                                }
                            } else {
                                StageSection::MidbossNonspell { seq: 0 }
                            }
                        } else {
                            StageSection::SecondHalf { seq: 0 }
                        }
                    }
                    4883..=6112 => StageSection::SecondHalf { seq: 0 },
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 72..=75) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 76..=79) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 80..=83) => StageSection::BossSpell {
                                    seq: 2,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 84..=87) => StageSection::BossSpell {
                                    seq: 3,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    2 => StageSection::BossNonspell { seq: 0 },
                                    1 => StageSection::BossNonspell { seq: 1 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            6 => StageLocation {
                stage: Stage::Six,
                section: match self.ecl_time {
                    0..=659 => StageSection::Start,
                    660..=1179 => StageSection::FirstHalf { seq: 0 },
                    1180..=1913 => StageSection::FirstHalf { seq: 1 },
                    1914..=2517 => {
                        if let Some(boss) = &self.boss_state {
                            if let Some(spell) = &boss.active_spell {
                                StageSection::MidbossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(spell.0),
                                }
                            } else {
                                StageSection::MidbossNonspell { seq: 0 }
                            }
                        } else {
                            StageSection::PreBoss
                        }
                    }
                    _ => {
                        if let Some(state) = &self.boss_state {
                            match state.active_spell.map(|x| x.0) {
                                Some(id @ 92..=95) => StageSection::BossSpell {
                                    seq: 0,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 96..=99) => StageSection::BossSpell {
                                    seq: 1,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 100..=103) => StageSection::BossSpell {
                                    seq: 2,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 104..=107) => StageSection::BossSpell {
                                    seq: 3,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 108..=111) => StageSection::BossSpell {
                                    seq: 4,
                                    spell: convert_to_spellcard!(id),
                                },
                                Some(id @ 112..=115) => StageSection::BossSpell {
                                    seq: 5,
                                    spell: convert_to_spellcard!(id),
                                },
                                None => match state.remaining_lifebars {
                                    4 => StageSection::BossNonspell { seq: 0 },
                                    3 => StageSection::BossNonspell { seq: 1 },
                                    2 => StageSection::BossNonspell { seq: 2 },
                                    1 => StageSection::BossNonspell { seq: 3 },
                                    _ => StageSection::Unknown,
                                },
                                _ => StageSection::Unknown,
                            }
                        } else {
                            StageSection::Unknown
                        }
                    }
                },
            },
            7 => StageLocation {
                stage: Stage::Extra,
                section: StageSection::Unknown,
            },
            8 => StageLocation {
                stage: Stage::Phantasm,
                section: StageSection::Unknown,
            },
            other => unreachable!("unknown stage {}", other),
        }
    }
}

mod serialize_time {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use time::OffsetDateTime;

    pub fn serialize<S: Serializer>(time: &OffsetDateTime, ser: S) -> Result<S::Ok, S::Error> {
        time.unix_timestamp().serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<OffsetDateTime, D::Error> {
        OffsetDateTime::from_unix_timestamp(i64::deserialize(de)?)
            .map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    StartGame {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        character: ShotType<Touhou7>,
        difficulty: Difficulty,
        location: StageLocation,
        practice: bool,
        lives: u8,
        bombs: u8,
        power: u8,
    },
    EndGame {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
        misses: u32,
        bombs: u32,
        continues: u8,
        cleared: bool,
    },
    StageCleared {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        stage: Stage,
    },
    EnterSection {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
        lives: u8,
        bombs: u8,
        power: u8,
        continues: u8,
    },
    Extend {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
    },
    Miss {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
    },
    Bomb {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
    },
    FinishSpell {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        spell: SpellCard<Touhou7>,
        captured: bool,
    },
    BorderStart {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
    },
    BorderEnd {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
        location: StageLocation,
        broken: bool,
    },
    Pause {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
    },
    Unpause {
        #[serde(with = "serialize_time")]
        time: OffsetDateTime,
    },
}

impl GameEvent {
    pub fn time(&self) -> OffsetDateTime {
        match self {
            Self::StartGame { time, .. }
            | Self::EndGame { time, .. }
            | Self::EnterSection { time, .. }
            | Self::StageCleared { time, .. }
            | Self::Extend { time, .. }
            | Self::Miss { time, .. }
            | Self::Bomb { time, .. }
            | Self::FinishSpell { time, .. }
            | Self::BorderStart { time, .. }
            | Self::BorderEnd { time, .. }
            | Self::Pause { time, .. }
            | Self::Unpause { time, .. } => *time,
        }
    }
}

impl Display for GameEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartGame {
                character,
                location,
                practice,
                difficulty,
                ..
            } => {
                if *practice {
                    write!(
                        f,
                        "Started {} practice game as {} at {}",
                        difficulty, character, location
                    )
                } else {
                    write!(
                        f,
                        "Started {} game as {} at {}",
                        difficulty, character, location
                    )
                }
            }
            Self::EndGame {
                location,
                misses,
                bombs,
                continues,
                cleared,
                ..
            } => {
                write!(
                    f,
                    "{} game at {} with {} miss{}, {} bomb{}, and {} continue{} used",
                    if *cleared { "Cleared" } else { "Ended" },
                    location,
                    *misses,
                    if *misses != 1 { "es" } else { "" },
                    *bombs,
                    if *bombs != 1 { "s" } else { "" },
                    *continues,
                    if *continues != 1 { "s" } else { "" }
                )
            }
            Self::EnterSection {
                location,
                lives,
                bombs,
                power,
                continues,
                ..
            } => {
                write!(
                    f,
                    "Entering {} with {} {}, {} bomb{}, {} power, and {} continue{} used",
                    location,
                    *lives,
                    if *lives == 1 { "life" } else { "lives" },
                    *bombs,
                    if *bombs != 1 { "s" } else { "" },
                    *power,
                    *continues,
                    if *continues != 1 { "s" } else { "" }
                )
            }
            Self::StageCleared { stage, .. } => write!(f, "Cleared {}", stage),
            Self::Extend { location, .. } => write!(f, "Got extend at {}", location),
            Self::Miss { location, .. } => write!(f, "Missed at {}", location),
            Self::Bomb { location, .. } => write!(f, "Bombed at {}", location),
            Self::FinishSpell {
                spell,
                captured: capture,
                ..
            } => {
                write!(
                    f,
                    "{} #{:03} {}",
                    if *capture { "Captured" } else { "Failed" },
                    spell.id(),
                    spell.name()
                )
            }
            Self::BorderStart { location, .. } => {
                write!(f, "Border started at {}", location)
            }
            Self::BorderEnd {
                broken, location, ..
            } => {
                if *broken {
                    write!(f, "Border broken at {}", location)
                } else {
                    write!(f, "Border ended at {}", location)
                }
            }
            Self::Pause { .. } => f.write_str("Paused game"),
            Self::Unpause { .. } => f.write_str("Unpaused game"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventStream {
    location: Option<StageLocation>,
    last_states: Option<(bool, bool, PlayerState, StageState)>,
    last_border_start: Option<OffsetDateTime>,
    start_time: Option<OffsetDateTime>,
    init_read_time: Option<Instant>,
    events: VecDeque<GameEvent>,
}

impl EventStream {
    pub fn new() -> Self {
        Self::default()
    }

    fn boss_finished(location: &StageLocation, stage: &StageState) -> bool {
        location.is_end_spell()
            && stage
                .boss_state
                .map(|boss| boss.active_spell.is_none())
                .unwrap_or(true)
    }

    fn update_location(&mut self, new_location: StageLocation) -> (bool, StageLocation) {
        if !new_location.is_unknown() {
            let prev_location = self.location.replace(new_location);
            (prev_location != Some(new_location), new_location)
        } else {
            (false, self.location.unwrap_or(new_location))
        }
    }

    fn update_border(
        &mut self,
        active: bool,
        time: OffsetDateTime,
        location: StageLocation,
        stage: &StageState,
    ) {
        if active {
            if self.last_border_start.is_none() {
                self.last_border_start = Some(time);
                self.events
                    .push_back(GameEvent::BorderStart { time, location });
            }
        } else if let Some(duration) = self
            .last_border_start
            .take()
            .and_then(|start| (time - start).max(time::Duration::ZERO).try_into().ok())
        {
            // don't treat border as broken if it happens at end of stage or before a boss fight
            let duration: Duration = duration;
            let broken = (duration <= Duration::from_millis(8925))
                && (location.section != StageSection::PreBoss)
                && !Self::boss_finished(&location, stage);

            self.events.push_back(GameEvent::BorderEnd {
                time,
                broken,
                location,
            });
        }
    }

    fn finish_spell(
        &mut self,
        time: OffsetDateTime,
        spell: SpellCard<Touhou7>,
        captured: bool,
        location: &StageLocation,
        stage: &StageState,
    ) {
        self.events.push_back(GameEvent::FinishSpell {
            time,
            spell,
            captured,
        });

        if Self::boss_finished(location, stage) {
            self.events.push_back(GameEvent::StageCleared {
                time,
                stage: location.stage,
            });
        }
    }

    fn end_game(
        &mut self,
        mut cleared: bool,
        time: OffsetDateTime,
        practice: bool,
        player: PlayerState,
        stage: StageState,
    ) {
        let (_, location) = self.update_location(stage.resolve_stage_section());

        self.update_border(false, time, location, &stage);

        if practice && Self::boss_finished(&location, &stage) {
            cleared = true;
        }

        self.events.push_back(GameEvent::EndGame {
            time,
            location,
            misses: player.total_misses,
            bombs: player.total_bombs,
            continues: player.continues,
            cleared,
        });

        self.last_states = None;
        self.last_border_start = None;
        self.location = None;
        self.start_time = None;
        self.init_read_time = None;
    }

    pub fn update(&mut self, proc: &Touhou7Memory) -> IOResult<()> {
        let time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        match proc.get_game_state()? {
            GameState::InGame {
                practice,
                paused,
                stage,
                player,
                ..
            } => {
                let now_instant = Instant::now();
                let values_are_stable = if let Some(d) = self
                    .init_read_time
                    .and_then(|init_time| now_instant.checked_duration_since(init_time))
                {
                    d >= Duration::from_secs(1)
                } else {
                    self.init_read_time = Some(now_instant);
                    false
                };

                if !values_are_stable {
                    sleep(Duration::from_secs(1));
                    return self.update(proc);
                }

                if let Some((_, prev_paused, prev_player, prev_stage)) =
                    self.last_states.replace((practice, paused, player, stage))
                {
                    match (prev_paused, paused) {
                        (false, true) => self.events.push_back(GameEvent::Pause { time }),
                        (true, false) => self.events.push_back(GameEvent::Unpause { time }),
                        _ => {}
                    }

                    let (location_updated, location) =
                        self.update_location(stage.resolve_stage_section());

                    if player.border_active != prev_player.border_active {
                        self.update_border(player.border_active, time, location, &stage);
                    }

                    if player.total_misses == (prev_player.total_misses + 1) {
                        self.events.push_back(GameEvent::Miss { time, location });
                    }

                    if player.total_bombs == (prev_player.total_bombs + 1) {
                        self.events.push_back(GameEvent::Bomb { time, location });
                    }

                    if player.lives == (prev_player.lives + 1) {
                        self.events.push_back(GameEvent::Extend { time, location });
                    }

                    let cur_boss_spell = stage
                        .boss_state
                        .as_ref()
                        .and_then(|x| x.active_spell.as_ref())
                        .map(|x| (convert_to_spellcard!(x.0), x.1));

                    let prev_boss_spell = prev_stage
                        .boss_state
                        .as_ref()
                        .and_then(|x| x.active_spell.as_ref())
                        .map(|x| (convert_to_spellcard!(x.0), x.1));

                    match (prev_boss_spell, cur_boss_spell) {
                        (Some(prev), Some(cur)) => {
                            if prev.0 != cur.0 {
                                self.finish_spell(time, prev.0, prev.1, &location, &stage);
                            }
                        }
                        (Some(prev), None) => {
                            self.finish_spell(time, prev.0, prev.1, &location, &stage)
                        }
                        _ => {}
                    }

                    if location_updated {
                        self.events.push_back(GameEvent::EnterSection {
                            time,
                            location,
                            lives: player.lives,
                            bombs: player.bombs,
                            power: player.power,
                            continues: player.continues,
                        });
                    }
                } else {
                    let location = stage.resolve_stage_section();
                    self.location = Some(location);
                    self.start_time = Some(time);
                    self.events.push_back(GameEvent::StartGame {
                        time,
                        location,
                        practice,
                        difficulty: stage.difficulty,
                        character: player.character,
                        lives: player.lives,
                        bombs: player.bombs,
                        power: player.power,
                    });

                    if paused {
                        self.events.push_back(GameEvent::Pause { time });
                    }
                }
            }
            GameState::LoadingStage => {}
            GameState::GameOver { cleared } => {
                if let Some((practice, _, _, _)) = self.last_states.take() {
                    self.end_game(
                        cleared,
                        time,
                        practice,
                        proc.get_player_state()?,
                        proc.get_stage_state()?,
                    )
                }
            }
            GameState::Unknown { state, mode } => {
                eprintln!("observed unknown state {}/{}", state, mode)
            }
            _ => {
                if let Some((practice, _, player, stage)) = self.last_states.take() {
                    self.end_game(false, time, practice, player, stage)
                }
            }
        }

        Ok(())
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = GameEvent> + '_ {
        self.events.drain(..)
    }
}

fn watcher(window: Window) {
    let mut system = System::new();
    let mut cur_process: Option<Touhou7Memory> = None;
    let mut event_stream = EventStream::new();

    window.emit("game-detached", ()).unwrap();

    loop {
        if let Some(th_proc) = cur_process.as_ref() {
            if !th_proc.is_running(&mut system) {
                window.emit("game-detached", ()).unwrap();
                cur_process = None;
                continue;
            }

            if let Err(e) = event_stream.update(th_proc) {
                window.emit("error", e.to_string()).unwrap();
                continue;
            }

            for event in event_stream.drain_events() {
                window.emit("game-event", event).unwrap();
            }

            sleep(Duration::from_millis(50));
        } else {
            system.refresh_processes_specifics(ProcessRefreshKind::new());
            cur_process = Touhou7Memory::new_autodetect(&system)
                .expect("could not initialize TH07 memory reader");

            if let Some(proc) = &cur_process {
                window.emit("game-attached", proc.pid()).unwrap();
                event_stream = EventStream::new();
            }

            sleep(Duration::from_secs(1))
        }
    }
}

#[tauri::command]
fn format_spellcard(spell: SpellCard<Touhou7>) -> &'static str {
    spell.name()
}

#[tauri::command]
fn init_events(window: Window) {
    std::thread::spawn(move || watcher(window));
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![format_spellcard, init_events])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
