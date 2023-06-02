use std::fmt::Display;

use crate::types::{iterable_enum, Character, IterableEnum};
use anyhow::anyhow;

pub mod replay;
pub mod score;
pub mod spellcard_names;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Extra,
    Phantasm,
}

iterable_enum!(
    Stage,
    StageEnumIter,
    [0, Stage::One],
    [1, Stage::Two],
    [2, Stage::Three],
    [3, Stage::Four],
    [4, Stage::Five],
    [5, Stage::Six],
    [6, Stage::Extra],
    [7, Stage::Phantasm]
);

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::One => "Stage 1",
            Self::Two => "Stage 2",
            Self::Three => "Stage 3",
            Self::Four => "Stage 4",
            Self::Five => "Stage 5",
            Self::Six => "Stage 6",
            Self::Extra => "Extra Stage",
            Self::Phantasm => "Phantasm Stage",
        })
    }
}

impl TryFrom<u8> for Stage {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::One),
            1 => Ok(Self::Two),
            2 => Ok(Self::Three),
            3 => Ok(Self::Four),
            4 => Ok(Self::Five),
            5 => Ok(Self::Six),
            6 => Ok(Self::Extra),
            7 => Ok(Self::Phantasm),
            _ => Err(anyhow!(
                "invalid stage type {} (valid types are 0-7)",
                value
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StageProgress {
    NotStarted,
    LostAt(Stage),
    AllClear,
}

impl TryFrom<u8> for StageProgress {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NotStarted),
            1 => Ok(Self::LostAt(Stage::One)),
            2 => Ok(Self::LostAt(Stage::Two)),
            3 => Ok(Self::LostAt(Stage::Three)),
            4 => Ok(Self::LostAt(Stage::Four)),
            5 => Ok(Self::LostAt(Stage::Five)),
            6 => Ok(Self::LostAt(Stage::Six)),
            7 => Ok(Self::LostAt(Stage::Extra)),
            8 => Ok(Self::LostAt(Stage::Phantasm)),
            99 => Ok(Self::AllClear),
            _ => Err(anyhow!("invalid stage progress value {}", value)),
        }
    }
}

impl Display for StageProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => f.write_str("Not Started"),
            Self::LostAt(s) => s.fmt(f),
            Self::AllClear => f.write_str("All Clear"),
        }
    }
}

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
