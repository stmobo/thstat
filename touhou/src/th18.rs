use std::fmt::Display;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

use crate::types::shot_type::InvalidShotType;
use crate::types::{
    iterable_enum, Character, Game, GameId, InvalidCardId, ShotType as WrappedShot, ShotTypeId,
    SpellCardId, SpellCardInfo,
};

pub mod score;
pub mod spellcards;

use spellcards::SPELL_CARDS;

#[derive(Debug, Clone, Copy)]
pub enum ShotType {
    Reimu,
    Marisa,
    Sakuya,
    Sanae,
}

impl ShotType {
    pub fn character(self) -> Character {
        match self {
            Self::Reimu => Character::Reimu,
            Self::Marisa => Character::Marisa,
            Self::Sakuya => Character::Sakuya,
            Self::Sanae => Character::Sanae,
        }
    }
}

impl From<ShotType> for u8 {
    fn from(value: ShotType) -> Self {
        match value {
            ShotType::Reimu => 0,
            ShotType::Marisa => 1,
            ShotType::Sakuya => 2,
            ShotType::Sanae => 3,
        }
    }
}

impl From<ShotType> for u16 {
    fn from(value: ShotType) -> Self {
        <ShotType as Into<u8>>::into(value) as u16
    }
}

impl TryFrom<u16> for ShotType {
    type Error = InvalidShotType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShotType::Reimu),
            1 => Ok(ShotType::Marisa),
            2 => Ok(ShotType::Sakuya),
            3 => Ok(ShotType::Sanae),
            _ => Err(InvalidShotType::InvalidShotId(value, 0, 3)),
        }
    }
}

impl TryFrom<u8> for ShotType {
    type Error = InvalidShotType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        <ShotType as TryFrom<u16>>::try_from(value as u16)
    }
}

impl Display for ShotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            ShotType::Reimu => "Reimu",
            ShotType::Marisa => "Marisa",
            ShotType::Sakuya => "Sakuya",
            ShotType::Sanae => "Sanae",
        })
    }
}

iterable_enum!(
    ShotType,
    ShotTypeIter,
    [0, ShotType::Reimu],
    [1, ShotType::Marisa],
    [2, ShotType::Sakuya],
    [3, ShotType::Sanae]
);

// impl ShotTypeId for ShotType {
//     fn fmt_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.fmt(f)
//     }

//     fn game_id(&self) -> GameId {
//         GameId::UM
//     }

//     fn raw_id(&self) -> u16 {
//         (*self).into()
//     }

//     fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
//         if game == GameId::UM {
//             id.try_into()
//         } else {
//             Err(InvalidShotType::UnexpectedGameId(game, GameId::PCB))
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellId(NonZeroU16);

impl From<SpellId> for u32 {
    fn from(value: SpellId) -> Self {
        value.0.get() as u32
    }
}

// impl TryFrom<u32> for SpellId {
//     type Error = InvalidCardId;

//     fn try_from(value: u32) -> Result<Self, Self::Error> {
//         if let Ok(Some(value)) = <u16 as TryFrom<u32>>::try_from(value).map(NonZeroU16::new) {
//             if value.get() <= (SPELL_CARDS.len() as u16) {
//                 return Ok(Self(value));
//             }
//         }

//         Err(InvalidCardId::InvalidCard(
//             GameId::UM,
//             value,
//             SPELL_CARDS.len() as u32,
//         ))
//     }
// }

impl Display for SpellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// impl SpellCardId for SpellId {
//     fn card_info(&self) -> &'static SpellCardInfo {
//         &SPELL_CARDS[(self.0.get() - 1) as usize]
//     }

//     fn game_id(&self) -> GameId {
//         GameId::UM
//     }

//     fn raw_id(&self) -> u32 {
//         (*self).into()
//     }

//     fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
//         if game == GameId::UM {
//             id.try_into()
//         } else {
//             Err(InvalidCardId::UnexpectedGameId(game, GameId::PCB))
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou18;

#[cfg(feature = "find-process")]
impl Touhou18 {
    pub fn find_process(system: &System) -> Option<&Process> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find(|&process| {
                process
                    .exe()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("th18"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}

// impl Game for Touhou18 {
//     type SpellID = SpellId;
//     type ShotTypeID = ShotType;
//     type SpellCardRecord = SpellCardData;
//     type PracticeRecord = PracticeData;
//     type ScoreFile = ScoreFile;

//     fn game_id(&self) -> GameId {
//         GameId::UM
//     }

//     fn score_path(&self) -> &Path {
//         &self.score_path
//     }

//     fn load_score_file<R: std::io::Read>(&self, src: R) -> Result<Self::ScoreFile, anyhow::Error> {
//         ScoreFile::new(src)
//     }
// }
