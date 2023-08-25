use std::collections::VecDeque;
use std::fmt::{Display, Write};
use std::io;
use std::io::{ErrorKind, Result as IOResult};
use std::thread::sleep;
use std::time::{Duration, Instant};

use process_memory::{
    Architecture, DataMember, Memory, ProcessHandle, ProcessHandleExt, TryIntoProcessHandle,
};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};
use time::OffsetDateTime;
use touhou::{ShotType, SpellCard, Stage, Touhou7};

#[derive(Debug, Clone)]
pub struct Touhou7Memory {
    pid: u32,
    stage: DataMember<u32>,
    game_state: DataMember<u32>,
    game_mode: DataMember<u8>,
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
            game_state: DataMember::new_offset(handle, vec![0x00575aa8]),
            game_mode: DataMember::new_offset(handle, vec![0x0062f648]),
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
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_running(&self, system: &mut System) -> bool {
        system.refresh_process_specifics(Pid::from_u32(self.pid), ProcessRefreshKind::new())
    }

    pub fn get_game_state(&self) -> IOResult<GameState> {
        unsafe { self.game_state.read().map(GameState::from) }
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

            Ok(StageState {
                stage: self.stage.read()?,
                mode: self.game_mode.read().map(GameMode)?,
                ecl_time: self.ecl_time.read()?,
                boss_state,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    InMenu,
    InGame,
    GameOver,
    Unknown(u32),
}

impl From<u32> for GameState {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::InMenu,
            2 => Self::InGame,
            6 | 7 => Self::GameOver,
            other => Self::Unknown(other),
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InMenu => f.write_str("In Menu"),
            Self::InGame => f.write_str("In-Game"),
            Self::GameOver => f.write_str("Game Over"),
            Self::Unknown(value) => write!(f, "Unknown state {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GameMode(u8);

impl GameMode {
    pub fn is_practice(&self) -> bool {
        (self.0 & 1) != 0
    }

    pub fn is_demo(&self) -> bool {
        (self.0 & 2) != 0
    }

    pub fn is_paused(&self) -> bool {
        (self.0 & 4) == 0 // bit is set if UNpaused
    }

    pub fn is_replay(&self) -> bool {
        (self.0 & 8) != 0
    }

    pub fn game_ended(&self) -> bool {
        (self.0 & 0x10) != 0
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        if self.is_practice() {
            f.write_str(" Practice")?;
        }

        if self.is_demo() {
            f.write_str(" Demo")?;
        }

        if self.is_replay() {
            f.write_str(" Replay")?;
        }

        if self.is_paused() {
            f.write_str(" Paused")?;
        }

        if self.game_ended() {
            f.write_str(" Ended")?;
        }

        f.write_str(" ]")
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
}

#[derive(Debug, Clone, Copy)]
pub struct BossState {
    id: u8,
    is_midboss: bool,
    remaining_lifebars: u32,
    active_spell: Option<(u32, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "section", rename_all = "snake_case")]
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
    mode: GameMode,
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
            3 => StageLocation {
                stage: Stage::Three,
                section: match self.ecl_time {
                    0..=389 => StageSection::Start,
                    390..=820 => StageSection::FirstHalf { seq: 0 },
                    821..=853 => StageSection::MidbossNonspell { seq: 0 },
                    854..=1804 => StageSection::SecondHalf { seq: 0 },
                    1805..=1857 => StageSection::SecondHalf { seq: 1 },
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
                            StageSection::SecondHalf { seq: 2 }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    StartSession {
        time: OffsetDateTime,
        character: ShotType<Touhou7>,
        location: StageLocation,
        practice: bool,
        lives: u8,
        bombs: u8,
        power: u8,
    },
    EndSession {
        time: OffsetDateTime,
        location: StageLocation,
        misses: u32,
        bombs: u32,
        continues: u8,
    },
    EnterSection {
        time: OffsetDateTime,
        location: StageLocation,
        lives: u8,
        bombs: u8,
        power: u8,
        continues: u8,
    },
    Extend {
        time: OffsetDateTime,
        location: StageLocation,
    },
    Miss {
        time: OffsetDateTime,
        location: StageLocation,
    },
    Bomb {
        time: OffsetDateTime,
        location: StageLocation,
    },
    FinishSpell {
        time: OffsetDateTime,
        spell: SpellCard<Touhou7>,
        captured: bool,
    },
}

impl Display for GameEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartSession {
                time,
                character,
                location,
                practice,
                lives: _,
                bombs: _,
                power: _,
            } => {
                if *practice {
                    write!(
                        f,
                        "[{}] Started practice session as {} at {}",
                        time, character, location
                    )
                } else {
                    write!(f, "Started run as {} at {}", character, location)
                }
            }
            Self::EndSession {
                time,
                location,
                misses,
                bombs,
                continues,
            } => {
                write!(
                    f,
                    "[{}] Ended session at {} with {} miss{}, {} bomb{}, and {} continue{} used",
                    time,
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
                time,
                location,
                lives,
                bombs,
                power,
                continues,
            } => {
                write!(
                    f,
                    "[{}] Entering {} with {} {}, {} bomb{}, {} power, and {} continue{} used",
                    time,
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
            Self::Extend { time, location } => write!(f, "[{}] Got extend at {}", time, location),
            Self::Miss { time, location } => write!(f, "[{}] Missed at {}", time, location),
            Self::Bomb { time, location } => write!(f, "[{}] Bombed at {}", time, location),
            Self::FinishSpell {
                time,
                spell,
                captured: capture,
            } => {
                write!(
                    f,
                    "[{}] {} #{:03} {}",
                    time,
                    if *capture { "Captured" } else { "Failed" },
                    spell.id(),
                    spell.name()
                )
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventStream {
    last_state: Option<(PlayerState, StageState)>,
    events: VecDeque<GameEvent>,
    game_init_time: Option<Instant>,
}

impl EventStream {
    pub fn new() -> Self {
        Self {
            last_state: None,
            game_init_time: None,
            events: VecDeque::new(),
        }
    }

    fn get_state(proc: &Touhou7Memory) -> IOResult<Option<(PlayerState, StageState)>> {
        if matches!(proc.get_game_state()?, GameState::InGame) {
            let stage = proc.get_stage_state()?;

            if stage.mode.is_replay() {
                return Ok(None);
            }

            let player = proc.get_player_state()?;
            Ok(Some((player, stage)))
        } else {
            Ok(None)
        }
    }

    pub fn update(&mut self, proc: &Touhou7Memory) -> IOResult<()> {
        let cur_state = proc.get_game_state()?;
        if matches!(cur_state, GameState::InGame) {
            let cur_time = Instant::now();
            let values_are_stable = if let Some(init_time) = self.game_init_time {
                cur_time
                    .checked_duration_since(init_time)
                    .map(|d| d >= Duration::from_secs(1))
                    .unwrap_or(false)
            } else {
                self.game_init_time = Some(cur_time);
                false
            };

            if !values_are_stable {
                sleep(Duration::from_secs(1));
                return self.update(proc);
            }
        }

        let time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        if let Some((player, stage)) = Self::get_state(proc)? {
            let location = stage.resolve_stage_section();

            if let Some((prev_player, prev_stage)) = self.last_state.replace((player, stage)) {
                let prev_location = prev_stage.resolve_stage_section();

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
                            self.events.push_back(GameEvent::FinishSpell {
                                time,
                                spell: prev.0,
                                captured: prev.1,
                            });
                        }
                    }
                    (Some(prev), None) => self.events.push_back(GameEvent::FinishSpell {
                        time,
                        spell: prev.0,
                        captured: prev.1,
                    }),
                    _ => {}
                }

                if location != prev_location && !location.is_unknown() {
                    self.events.push_back(GameEvent::EnterSection {
                        time,
                        location,
                        lives: player.lives,
                        bombs: player.bombs,
                        power: player.power,
                        continues: player.continues,
                    })
                }
            } else {
                self.events.push_back(GameEvent::StartSession {
                    time,
                    character: player.character,
                    location: stage.resolve_stage_section(),
                    practice: stage.mode.is_practice(),
                    lives: player.lives,
                    bombs: player.bombs,
                    power: player.power,
                })
            }
        } else if let Some((player, stage)) = self.last_state.take() {
            let (end_player, end_stage) = if matches!(proc.get_game_state()?, GameState::GameOver) {
                let end_stage = proc.get_stage_state()?;
                if !end_stage.mode.is_replay() {
                    if let Some(boss_spell) = end_stage.boss_state.and_then(|x| x.active_spell) {
                        self.events.push_back(GameEvent::FinishSpell {
                            time,
                            spell: convert_to_spellcard!(boss_spell.0),
                            captured: boss_spell.1,
                        });
                    }

                    (proc.get_player_state()?, end_stage)
                } else {
                    (player, stage)
                }
            } else {
                (player, stage)
            };

            self.events.push_back(GameEvent::EndSession {
                time,
                location: end_stage.resolve_stage_section(),
                misses: end_player.total_misses,
                bombs: end_player.total_bombs,
                continues: end_player.continues,
            });

            self.game_init_time = None;
        }

        Ok(())
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = GameEvent> + '_ {
        self.events.drain(..)
    }
}

fn main() {
    let mut system = System::new();
    let mut cur_process: Option<Touhou7Memory> = None;
    let mut event_stream = EventStream::new();

    loop {
        if let Some(th_proc) = cur_process.as_ref() {
            if !th_proc.is_running(&mut system) {
                cur_process = None;
                continue;
            }

            event_stream
                .update(th_proc)
                .expect("could not read process memory");

            for event in event_stream.drain_events() {
                println!("{}", event);
            }

            sleep(Duration::from_millis(500));
        } else {
            println!("Waiting for PCB process...");

            system.refresh_processes_specifics(ProcessRefreshKind::new());
            cur_process = Touhou7Memory::new_autodetect(&system)
                .expect("could not initialize TH07 memory reader");

            sleep(Duration::from_secs(1))
        }
    }
}
