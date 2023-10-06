//! Types and traits for reading score file data.

mod crypt;
mod decompress;

use std::fmt::{Debug, Display};
use std::io::{self, ErrorKind, Read};
use std::str;
use std::str::FromStr;

use anyhow::anyhow;
pub use crypt::ThCrypt;
pub use decompress::StreamDecompressor;

use crate::types::{Difficulty, Game, ShotType, SpellCard, Stage};

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

/// A type representing a spell card record stored within a score file.
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

/// A type representing a spell card practice record stored within a score file, for games that have separate spell practice modes (such as Touhou 8).
pub trait SpellPracticeRecord<G: Game>: SpellCardRecord<G> {
    fn practice_attempts(&self, shot: &ShotType<G>) -> u32;
    fn practice_captures(&self, shot: &ShotType<G>) -> u32;
    fn practice_max_bonus(&self, shot: &ShotType<G>) -> u32;

    fn practice_total_attempts(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.practice_attempts(shot))
            .sum()
    }

    fn practice_total_captures(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.practice_captures(shot))
            .sum()
    }

    fn practice_total_max_bonus(&self) -> u32 {
        self.shot_types()
            .iter()
            .map(|shot| self.practice_max_bonus(shot))
            .max()
            .unwrap()
    }
}

/// A type representing a stage practice record stored within a score file.
pub trait PracticeRecord<G: Game>: Sized + Debug {
    fn high_score(&self) -> u32;
    fn attempts(&self) -> u32;
    fn shot_type(&self) -> ShotType<G>;
    fn difficulty(&self) -> Difficulty<G>;
    fn stage(&self) -> Stage<G>;
}

/// A type representing a loaded score file.
pub trait ScoreFile<G: Game>: Sized + Debug {
    type SpellCardRecord: SpellCardRecord<G>;
    type PracticeRecord: PracticeRecord<G>;

    fn spell_cards(&self) -> &[Self::SpellCardRecord];
    fn practice_records(&self) -> &[Self::PracticeRecord];
}
