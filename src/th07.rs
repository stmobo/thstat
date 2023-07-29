use std::fmt::Display;

use byteorder::ReadBytesExt;
use sysinfo::{Process, ProcessExt, System, SystemExt};

use self::score::{PracticeData, ScoreFile, ScoreReader, SpellCardData};
use crate::types::shot_type::InvalidShotType;
use crate::types::{iterable_enum, Character, Game, GameId, IterableEnum, SpellCardInfo};

pub mod replay;
pub mod score;
pub mod spellcards;

use spellcards::SPELL_CARDS;

#[derive(Debug, Clone, Copy)]
pub enum ShotType {
    ReimuA,
    ReimuB,
    MarisaA,
    MarisaB,
    SakuyaA,
    SakuyaB,
}

impl ShotType {
    pub fn character(self) -> Character {
        match self {
            Self::ReimuA | Self::ReimuB => Character::Reimu,
            Self::MarisaA | Self::MarisaB => Character::Marisa,
            Self::SakuyaA | Self::SakuyaB => Character::Sakuya,
        }
    }

    pub fn is_type_a(self) -> bool {
        matches!(self, Self::ReimuA | Self::MarisaA | Self::SakuyaA)
    }

    pub fn is_type_b(self) -> bool {
        !self.is_type_a()
    }
}

impl From<ShotType> for u8 {
    fn from(value: ShotType) -> Self {
        match value {
            ShotType::ReimuA => 0,
            ShotType::ReimuB => 1,
            ShotType::MarisaA => 2,
            ShotType::MarisaB => 3,
            ShotType::SakuyaA => 4,
            ShotType::SakuyaB => 5,
        }
    }
}

impl TryFrom<u8> for ShotType {
    type Error = InvalidShotType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShotType::ReimuA),
            1 => Ok(ShotType::ReimuB),
            2 => Ok(ShotType::MarisaA),
            3 => Ok(ShotType::MarisaB),
            4 => Ok(ShotType::SakuyaA),
            5 => Ok(ShotType::SakuyaB),
            _ => Err(InvalidShotType::new(value, 0, 5)),
        }
    }
}

impl Display for ShotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            ShotType::ReimuA => "Reimu A",
            ShotType::ReimuB => "Reimu B",
            ShotType::MarisaA => "Marisa A",
            ShotType::MarisaB => "Marisa B",
            ShotType::SakuyaA => "Sakuya A",
            ShotType::SakuyaB => "Sakuya B",
        })
    }
}

iterable_enum!(
    ShotType,
    ShotTypeIter,
    [0, ShotType::ReimuA],
    [1, ShotType::ReimuB],
    [2, ShotType::MarisaA],
    [3, ShotType::MarisaB],
    [4, ShotType::SakuyaA],
    [5, ShotType::SakuyaB]
);

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
    const GAME_ID: GameId = GameId::PCB;
    const CARD_INFO: &'static [SpellCardInfo] = SPELL_CARDS;

    type ShotTypeInner = ShotType;
    type SpellCardRecord = SpellCardData;
    type PracticeRecord = PracticeData;
    type ScoreFile = ScoreFile;

    fn game_id() -> GameId {
        GameId::PCB
    }

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

    fn find_score_file(system: &System) -> Option<std::path::PathBuf> {
        Self::find_process(system).map(|proc| proc.exe().with_file_name("score.dat"))
    }

    fn load_score_file<R: std::io::Read>(src: R) -> Result<ScoreFile, anyhow::Error> {
        ScoreFile::new(src)
    }
}
