use std::fmt::Display;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sysinfo::{Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

use crate::types::shot_type::InvalidShotType;
use crate::types::{
    iterable_enum, Character, Game, GameId, InvalidCardId, IterableEnum, ShotType as WrappedShot,
    ShotTypeId, SpellCardId, SpellCardInfo,
};

pub mod replay;
pub mod score;
pub mod spellcards;

pub use score::{PracticeData, ScoreFile, SpellCardData};
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

impl From<ShotType> for u16 {
    fn from(value: ShotType) -> Self {
        <ShotType as Into<u8>>::into(value) as u16
    }
}

impl TryFrom<u16> for ShotType {
    type Error = InvalidShotType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ShotType::ReimuA),
            1 => Ok(ShotType::ReimuB),
            2 => Ok(ShotType::MarisaA),
            3 => Ok(ShotType::MarisaB),
            4 => Ok(ShotType::SakuyaA),
            5 => Ok(ShotType::SakuyaB),
            _ => Err(InvalidShotType::InvalidShotId(value, 0, 5)),
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

impl ShotTypeId for ShotType {
    fn fmt_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidShotType::UnexpectedGameId(game, GameId::PCB))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpellId(NonZeroU16);

impl From<SpellId> for u32 {
    fn from(value: SpellId) -> Self {
        value.0.get() as u32
    }
}

impl TryFrom<u32> for SpellId {
    type Error = InvalidCardId;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Ok(Some(value)) = <u16 as TryFrom<u32>>::try_from(value).map(NonZeroU16::new) {
            if value.get() <= (SPELL_CARDS.len() as u16) {
                return Ok(Self(value));
            }
        }

        Err(InvalidCardId::InvalidCard(
            GameId::PCB,
            value,
            SPELL_CARDS.len() as u32,
        ))
    }
}

impl Display for SpellId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SpellCardId for SpellId {
    fn card_info(&self) -> &'static SpellCardInfo {
        &SPELL_CARDS[(self.0.get() - 1) as usize]
    }

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn raw_id(&self) -> u32 {
        (*self).into()
    }

    fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId> {
        if game == GameId::PCB {
            id.try_into()
        } else {
            Err(InvalidCardId::UnexpectedGameId(game, GameId::PCB))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou7 {
    score_path: PathBuf,
}

impl Touhou7 {
    pub const SHOT_TYPES: &[WrappedShot<Touhou7>; 6] = &[
        WrappedShot::new(ShotType::ReimuA),
        WrappedShot::new(ShotType::ReimuB),
        WrappedShot::new(ShotType::MarisaA),
        WrappedShot::new(ShotType::MarisaB),
        WrappedShot::new(ShotType::SakuyaA),
        WrappedShot::new(ShotType::SakuyaB),
    ];

    pub fn new(score_file: PathBuf) -> Self {
        Self {
            score_path: score_file,
        }
    }

    pub fn new_from_process(proc: &Process) -> Self {
        Self::new(Self::find_score_file(proc))
    }

    pub fn score_path(&self) -> &Path {
        &self.score_path
    }

    pub async fn wait_for_game() -> Self {
        let mut system = System::new();
        let score_file = loop {
            system.refresh_processes_specifics(ProcessRefreshKind::new());

            if let Some(score_path) = Self::find_process(&system).map(Self::find_score_file) {
                break score_path;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        };

        Self::new(score_file)
    }

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
                    .map(|s| s.starts_with("th07"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}

impl Game for Touhou7 {
    type SpellID = SpellId;
    type ShotTypeID = ShotType;
    type SpellCardRecord = SpellCardData;
    type PracticeRecord = PracticeData;
    type ScoreFile = ScoreFile;

    fn game_id(&self) -> GameId {
        GameId::PCB
    }

    fn score_path(&self) -> &Path {
        &self.score_path
    }

    fn load_score_file<R: std::io::Read>(&self, src: R) -> Result<Self::ScoreFile, anyhow::Error> {
        ScoreFile::new(src)
    }
}
