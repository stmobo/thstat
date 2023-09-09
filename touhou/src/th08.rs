use serde::{Deserialize, Serialize};
#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, System, SystemExt};
use touhou_macros::NumericEnum;

#[cfg(feature = "score-file")]
pub mod score;

mod spellcards;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "score-file")]
pub use score::ScoreFile;
pub use spellcards::SpellId;

use crate::types::{
    Game, GameId, GameValue, InvalidDifficultyId, InvalidShotType, InvalidStageId,
    ShotType as WrappedShot, SpellCardInfo,
};

fn invalid_shot_type(value: u64) -> InvalidShotType {
    InvalidShotType::InvalidShotId(value as u16, 0, 11)
}

#[derive(Debug, NumericEnum)]
#[error_type = "InvalidShotType"]
#[convert_error = "invalid_shot_type"]
pub enum ShotType {
    #[name = "Reimu & Yukari"]
    BarrierTeam = 0,
    #[name = "Marisa & Alice"]
    MagicTeam = 1,
    #[name = "Sakuya & Remilia"]
    ScarletTeam = 2,
    #[name = "Youmu & Yuyuko"]
    GhostTeam = 3,
    Reimu = 4,
    Yukari = 5,
    Marisa = 6,
    Alice = 7,
    Sakuya = 8,
    Remilia = 9,
    Youmu = 10,
    Yuyuko = 11,
}

impl GameValue for ShotType {
    type RawValue = u16;
    type ConversionError = InvalidShotType;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidShotType::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_difficulty(value: u64) -> InvalidDifficultyId {
    InvalidDifficultyId::InvalidDifficulty(GameId::IN, value as u16, 5)
}

#[derive(Debug, NumericEnum, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[error_type = "InvalidDifficultyId"]
#[convert_error = "invalid_difficulty"]
pub enum Difficulty {
    Easy = 0,
    Normal = 1,
    Hard = 2,
    Lunatic = 3,
    Extra = 4,
    LastWord = 5,
}

impl GameValue for Difficulty {
    type RawValue = u16;
    type ConversionError = InvalidDifficultyId;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidDifficultyId> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidDifficultyId::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

fn invalid_stage(value: u64) -> InvalidStageId {
    InvalidStageId::InvalidStage(GameId::IN, value as u16, 8)
}

#[derive(Debug, NumericEnum, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
#[error_type = "InvalidStageId"]
#[convert_error = "invalid_stage"]
pub enum Stage {
    #[name = "Stage 1"]
    One = 0,
    #[name = "Stage 2"]
    Two = 1,
    #[name = "Stage 3"]
    Three = 2,
    #[name = "Stage 4 Uncanny"]
    FourA = 3,
    #[name = "Stage 4 Powerful"]
    FourB = 4,
    #[name = "Stage 5"]
    Five = 5,
    #[name = "Final A"]
    FinalA = 6,
    #[name = "Final B"]
    FinalB = 7,
    #[name = "Extra Stage"]
    Extra = 8,
    #[name = "Last Word"]
    LastWord = 9,
}

impl GameValue for Stage {
    type RawValue = u16;
    type ConversionError = InvalidStageId;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn raw_id(&self) -> u16 {
        (*self).into()
    }

    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidStageId> {
        if game == GameId::IN {
            id.try_into()
        } else {
            Err(InvalidStageId::UnexpectedGameId(game, GameId::IN))
        }
    }

    fn name(&self) -> &'static str {
        self.name()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Touhou8;

impl Touhou8 {
    pub const SHOT_TYPES: &[WrappedShot<Touhou8>; 12] = &[
        WrappedShot::new(ShotType::BarrierTeam),
        WrappedShot::new(ShotType::MagicTeam),
        WrappedShot::new(ShotType::ScarletTeam),
        WrappedShot::new(ShotType::GhostTeam),
        WrappedShot::new(ShotType::Reimu),
        WrappedShot::new(ShotType::Yukari),
        WrappedShot::new(ShotType::Marisa),
        WrappedShot::new(ShotType::Alice),
        WrappedShot::new(ShotType::Sakuya),
        WrappedShot::new(ShotType::Remilia),
        WrappedShot::new(ShotType::Youmu),
        WrappedShot::new(ShotType::Yuyuko),
    ];
}

#[cfg(feature = "find-process")]
impl Touhou8 {
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
                    .map(|s| s.starts_with("th08"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}

impl Game for Touhou8 {
    type SpellID = SpellId;
    type ShotTypeID = ShotType;
    type DifficultyID = Difficulty;
    type StageID = Stage;

    fn game_id(&self) -> GameId {
        GameId::IN
    }

    fn card_info(id: SpellId) -> &'static SpellCardInfo<Self> {
        id.card_info()
    }
}
