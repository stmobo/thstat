use std::fmt::Display;

use anyhow::anyhow;
use byteorder::ReadBytesExt;
use sysinfo::{Process, ProcessExt, System, SystemExt};

use self::score::{ScoreReader, SpellCardData};
use crate::types::{iterable_enum, Character, Game, GameId, IterableEnum};

pub mod replay;
pub mod score;
pub mod spellcard_names;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShotType {
    character: Character,
    type_b: bool,
}

impl ShotType {
    pub fn character(&self) -> Character {
        self.character
    }

    pub fn is_type_a(&self) -> bool {
        !self.type_b
    }

    pub fn is_type_b(&self) -> bool {
        self.type_b
    }
}

impl From<ShotType> for u8 {
    fn from(value: ShotType) -> Self {
        match value {
            REIMU_A => 0,
            REIMU_B => 1,
            MARISA_A => 2,
            MARISA_B => 3,
            SAKUYA_A => 4,
            SAKUYA_B => 5,
        }
    }
}

impl TryFrom<u8> for ShotType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self {
                character: Character::Reimu,
                type_b: false,
            }),
            1 => Ok(Self {
                character: Character::Reimu,
                type_b: true,
            }),
            2 => Ok(Self {
                character: Character::Marisa,
                type_b: false,
            }),
            3 => Ok(Self {
                character: Character::Marisa,
                type_b: true,
            }),
            4 => Ok(Self {
                character: Character::Sakuya,
                type_b: false,
            }),
            5 => Ok(Self {
                character: Character::Sakuya,
                type_b: true,
            }),
            _ => Err(anyhow!(
                "invalid character type {} (valid types are 0-5)",
                value
            )),
        }
    }
}

impl Display for ShotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.character.name())?;
        if self.is_type_a() {
            f.write_str(" A")
        } else {
            f.write_str(" B")
        }
    }
}

iterable_enum!(
    ShotType,
    ShotTypeIter,
    [0, REIMU_A],
    [1, REIMU_B],
    [2, MARISA_A],
    [3, MARISA_B],
    [4, SAKUYA_A],
    [5, SAKUYA_B]
);

pub const REIMU_A: ShotType = ShotType {
    character: Character::Reimu,
    type_b: false,
};

pub const REIMU_B: ShotType = ShotType {
    character: Character::Reimu,
    type_b: true,
};

pub const MARISA_A: ShotType = ShotType {
    character: Character::Marisa,
    type_b: false,
};

pub const MARISA_B: ShotType = ShotType {
    character: Character::Marisa,
    type_b: true,
};

pub const SAKUYA_A: ShotType = ShotType {
    character: Character::Sakuya,
    type_b: false,
};

pub const SAKUYA_B: ShotType = ShotType {
    character: Character::Sakuya,
    type_b: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Touhou7;

impl Touhou7 {
    pub fn new() -> Self {
        Self
    }

    pub fn read_score_data<R: ReadBytesExt>(src: R) -> Result<ScoreReader<R>, anyhow::Error> {
        ScoreReader::new(src)
    }
}

impl Game for Touhou7 {
    type ShotType = ShotType;
    type SpellCardRecord = SpellCardData;

    fn game_id() -> GameId {
        GameId::PCB
    }

    fn find_process(system: &System) -> Option<&Process> {
        system.processes().iter().map(|(_, process)| process).find(|&process| process
                .exe()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("th07"))
                .unwrap_or(false))
    }

    fn find_score_file(system: &System) -> Option<std::path::PathBuf> {
        Self::find_process(system).map(|proc| proc.exe().with_file_name("score.dat"))
    }
}
