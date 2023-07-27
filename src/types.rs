
use std::fmt::{Debug, Display};
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use std::str;
use std::str::FromStr;

use anyhow::anyhow;

use sysinfo::{Process, System};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Lunatic,
    Extra,
    Phantasm,
}

impl TryFrom<u8> for Difficulty {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Easy),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Hard),
            3 => Ok(Self::Lunatic),
            4 => Ok(Self::Extra),
            5 => Ok(Self::Phantasm),
            _ => Err(anyhow!(
                "invalid difficulty type {} (valid types are 0-5)",
                value
            )),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameId {
    PCB,
}

impl TryFrom<u16> for GameId {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            7 => Ok(Self::PCB),
            v => Err(anyhow!("invalid game ID {}", v)),
        }
    }
}

pub trait Game: Sized + Copy {
    type ShotType: IterableEnum
        + TryFrom<u8>
        + Into<u8>
        + Eq
        + Copy
        + Display
        + Debug
        + Sync
        + Send
        + Unpin;
    type SpellCardRecord: SpellCardRecord<Self>;

    fn game_id() -> GameId;
    fn find_process(system: &System) -> Option<&Process>;
    fn find_score_file(system: &System) -> Option<PathBuf>;

    fn shot_types() -> <Self::ShotType as IterableEnum>::EnumIter {
        Self::ShotType::iter_all()
    }

    fn load_card_records<R: Read + 'static>(
        src: R,
    ) -> Result<<Self::SpellCardRecord as SpellCardRecord<Self>>::CardReader, anyhow::Error> {
        <Self::SpellCardRecord as SpellCardRecord<Self>>::load_records(src)
    }
}

pub trait SpellCardRecord<G: Game>: Sized {
    type CardReader: Iterator<Item = Result<Self, anyhow::Error>>;

    fn load_records<R: Read + 'static>(src: R) -> Result<Self::CardReader, anyhow::Error>;

    fn card_id(&self) -> u16;
    fn spell_name(&self) -> &'static str;
    fn attempts(&self, shot: &G::ShotType) -> u32;
    fn captures(&self, shot: &G::ShotType) -> u32;
    fn max_bonus(&self, shot: &G::ShotType) -> u32;

    fn total_attempts(&self) -> u32 {
        G::ShotType::iter_all()
            .map(|shot| self.attempts(&shot))
            .sum()
    }

    fn total_captures(&self) -> u32 {
        G::ShotType::iter_all()
            .map(|shot| self.captures(&shot))
            .sum()
    }

    fn total_max_bonus(&self) -> u32 {
        G::ShotType::iter_all()
            .map(|shot| self.max_bonus(&shot))
            .max()
            .unwrap()
    }
}

pub trait IterableEnum: Sized {
    type EnumIter: Iterator<Item = Self> + 'static;
    fn iter_all() -> Self::EnumIter;
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

pub(crate) use iterable_enum;
