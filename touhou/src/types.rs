use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{self, ErrorKind, Read};
use std::path::Path;
use std::str;
use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub mod any;
pub mod shot_type;
pub mod spell_card;

pub use any::{GameId, Touhou};
pub use shot_type::{InvalidShotType, ShotType};
pub use spell_card::{InvalidCardId, SpellCard, SpellCardInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShortDate {
    month: u8,
    day: u8,
}

impl ShortDate {
    pub fn read_from<R: Read>(src: &mut R) -> Result<Self, io::Error> {
        let mut buf = [0; 6];

        src.read_exact(&mut buf)?;
        str::from_utf8(&buf[..5])
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?
            .parse()
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }
}

impl Display for ShortDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}/{:02}", self.month, self.day)
    }
}

impl FromStr for ShortDate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((first, last)) = s.split_once('/') {
            let month = first.parse()?;
            let day = last.parse()?;
            Ok(ShortDate { month, day })
        } else {
            Err(anyhow!("could not parse short date {}", s))
        }
    }
}

macro_rules! impl_enum_sqlx_type {
    ($t:ty as $conv_as:ty) => {
        impl<DB> sqlx::Type<DB> for $t
        where
            DB: sqlx::Database,
            $conv_as: sqlx::Type<DB>,
        {
            fn type_info() -> <DB as sqlx::Database>::TypeInfo {
                <$conv_as as sqlx::Type<DB>>::type_info()
            }
        }

        impl<'q, DB> sqlx::Encode<'q, DB> for $t
        where
            DB: sqlx::Database,
            $conv_as: sqlx::Encode<'q, DB>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                let val: $conv_as = (*self).into();
                val.encode_by_ref(buf)
            }

            fn encode(
                self,
                buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                let val: $conv_as = self.into();
                val.encode(buf)
            }

            fn produces(&self) -> Option<<DB as sqlx::Database>::TypeInfo> {
                let val: $conv_as = (*self).into();
                <$conv_as as sqlx::Encode<'q, DB>>::produces(&val)
            }

            fn size_hint(&self) -> usize {
                let val: $conv_as = (*self).into();
                <$conv_as as sqlx::Encode<'q, DB>>::size_hint(&val)
            }
        }

        impl<'r, DB> sqlx::Decode<'r, DB> for $t
        where
            DB: sqlx::Database,
            $conv_as: sqlx::Decode<'r, DB>,
        {
            fn decode(
                value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let val: $conv_as = <$conv_as as sqlx::Decode<'r, DB>>::decode(value)?;
                val.try_into().map_err(|e| {
                    let b: Box<dyn std::error::Error + Sync + Send> = Box::new(e);
                    b
                })
            }
        }
    };
}

pub(super) use impl_enum_sqlx_type;

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid difficulty ID {0} (valid values are 0..=5)")]
pub struct InvalidDifficulty(u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Lunatic,
    Extra,
    Phantasm,
}

impl TryFrom<u8> for Difficulty {
    type Error = InvalidDifficulty;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Easy),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Hard),
            3 => Ok(Self::Lunatic),
            4 => Ok(Self::Extra),
            5 => Ok(Self::Phantasm),
            _ => Err(InvalidDifficulty(value)),
        }
    }
}

impl From<Difficulty> for u8 {
    fn from(value: Difficulty) -> Self {
        match value {
            Difficulty::Easy => 0,
            Difficulty::Normal => 1,
            Difficulty::Hard => 2,
            Difficulty::Lunatic => 3,
            Difficulty::Extra => 4,
            Difficulty::Phantasm => 5,
        }
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Hard => "Hard",
            Self::Lunatic => "Lunatic",
            Self::Extra => "Extra",
            Self::Phantasm => "Phantasm",
        })
    }
}

iterable_enum!(
    Difficulty,
    DifficultyEnumIter,
    [0, Difficulty::Easy],
    [1, Difficulty::Normal],
    [2, Difficulty::Hard],
    [3, Difficulty::Lunatic],
    [4, Difficulty::Extra],
    [5, Difficulty::Phantasm]
);

impl_enum_sqlx_type!(Difficulty as u8);

#[derive(Debug, Clone, Copy, Error)]
#[error("Invalid stage ID {0} (valid values are 0..=7)")]
pub struct InvalidStage(u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
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
    type Error = InvalidStage;

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
            _ => Err(InvalidStage(value)),
        }
    }
}

impl From<Stage> for u8 {
    fn from(value: Stage) -> Self {
        match value {
            Stage::One => 0,
            Stage::Two => 1,
            Stage::Three => 2,
            Stage::Four => 3,
            Stage::Five => 4,
            Stage::Six => 5,
            Stage::Extra => 6,
            Stage::Phantasm => 7,
        }
    }
}

impl_enum_sqlx_type!(Stage as u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StageProgress {
    NotStarted,
    LostAt(Stage),
    AllClear,
}

impl Display for StageProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => f.write_str("Not Started"),
            Self::LostAt(s) => <Stage as Display>::fmt(s, f),
            Self::AllClear => f.write_str("All Clear"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Character {
    Reimu,
    Marisa,
    Sakuya,
}

impl Character {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Reimu => "Reimu",
            Self::Marisa => "Marisa",
            Self::Sakuya => "Sakuya",
        }
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub trait SpellCardId: Debug + Copy + Sync + Send + Unpin + 'static {
    fn card_info(&self) -> &'static SpellCardInfo;
    fn game_id(&self) -> GameId;
    fn raw_id(&self) -> u32;
    fn from_raw(id: u32, game: GameId) -> Result<Self, InvalidCardId>;
}

pub trait ShotTypeId: Debug + Copy + Sync + Send + Unpin + 'static {
    fn fmt_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn game_id(&self) -> GameId;
    fn raw_id(&self) -> u16;
    fn from_raw(id: u16, game: GameId) -> Result<Self, InvalidShotType>;
}

pub trait Game: Sized + Sync + Send + Unpin + 'static {
    type SpellID: SpellCardId;
    type ShotTypeID: ShotTypeId;

    type ScoreFile: ScoreFile<Self>;
    type SpellCardRecord: SpellCardRecord<Self>;
    type PracticeRecord: PracticeRecord<Self>;

    fn game_id(&self) -> GameId;
    fn score_path(&self) -> &Path;
    fn load_score_file<R: Read>(&self, src: R) -> Result<Self::ScoreFile, anyhow::Error>;
}

pub trait SpellCardRecord<G: Game>: Sized + Debug {
    fn card(&self) -> SpellCard<G>;
    fn shot_types(&self) -> &[ShotType<G>];
    fn attempts(&self, shot: &ShotType<G>) -> u32;
    fn captures(&self, shot: &ShotType<G>) -> u32;
    fn max_bonus(&self, shot: &ShotType<G>) -> u32;

    fn total_attempts(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.attempts(shot))
            .sum()
    }

    fn total_captures(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.captures(shot))
            .sum()
    }

    fn total_max_bonus(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.max_bonus(shot))
            .max()
            .unwrap()
    }
}

pub trait PracticeRecord<G: Game>: Sized + Debug {
    fn high_score(&self) -> u32;
    fn attempts(&self) -> u32;
    fn shot_type(&self) -> ShotType<G>;
    fn difficulty(&self) -> Difficulty;
    fn stage(&self) -> Stage;
}

pub trait ScoreFile<G: Game>: Sized + Debug {
    fn spell_cards(&self) -> &[G::SpellCardRecord];
    fn practice_records(&self) -> &[G::PracticeRecord];
}

pub trait IterableEnum: Sized {
    type EnumIter: Iterator<Item = Self> + 'static;
    fn iter_all() -> Self::EnumIter;
}

macro_rules! impl_wrapper_traits {
    ($t:ident, $val_ty:ty, $wrapped_ty:ty) => {
        impl<G: Game> PartialEq for $t<G> {
            fn eq(&self, other: &Self) -> bool {
                let a: $val_ty = self.0.raw_id();
                let b: $val_ty = other.0.raw_id();
                a == b
            }
        }

        impl<G: Game> Eq for $t<G> {}

        impl<G: Game> PartialOrd for $t<G> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                let a: $val_ty = self.0.raw_id();
                let b: $val_ty = other.0.raw_id();
                a.partial_cmp(&b)
            }
        }

        impl<G: Game> Ord for $t<G> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let a: $val_ty = self.0.raw_id();
                let b: $val_ty = other.0.raw_id();
                a.cmp(&b)
            }
        }

        impl<G: Game> Hash for $t<G> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                let v: $val_ty = self.0.raw_id();
                v.hash(state)
            }
        }

        impl<G: Game> Clone for $t<G> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<G: Game> Copy for $t<G> {}

        #[derive(serde::Serialize, serde::Deserialize)]
        struct SerializedAs {
            game: $crate::types::GameId,
            id: $val_ty,
        }

        impl<G: Game> serde::Serialize for $t<G> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let serialized: (GameId, $val_ty) = (self.0.game_id(), self.0.raw_id());
                <(GameId, $val_ty) as serde::Serialize>::serialize(&serialized, serializer)
            }
        }

        impl<'de, G: Game> serde::Deserialize<'de> for $t<G> {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let deserialized: (GameId, $val_ty) =
                    <(GameId, $val_ty) as serde::Deserialize<'de>>::deserialize(deserializer)?;

                <$wrapped_ty>::from_raw(deserialized.1, deserialized.0)
                    .map(Self)
                    .map_err(<D::Error as serde::de::Error>::custom)
            }
        }
    };
}

macro_rules! iterable_enum {
    ($t:ty, $iter:ident, [ $key0:literal, $val0:expr ] $(, [ $key:literal, $val:expr ] )*) => {
        pub struct $iter(u8);

        impl Iterator for $iter {
            type Item = $t;

            fn next(&mut self) -> Option<$t> {
                match self.0 {
                    $key0 => { self.0 += 1; Some($val0) },
                    $( $key => { self.0 += 1; Some($val) } ),*
                    _ => None
                }
            }
        }

        impl IterableEnum for $t {
            type EnumIter = $iter;

            fn iter_all() -> $iter {
                $iter($key0)
            }
        }
    };
}

pub(super) use impl_wrapper_traits;
pub(crate) use iterable_enum;
use thiserror::Error;
