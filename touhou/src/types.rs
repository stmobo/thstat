use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::{self, ErrorKind, Read};
use std::str;
use std::str::FromStr;

use anyhow::anyhow;

pub mod difficulty;
pub mod errors;
pub mod game_id;
pub mod shot_type;
pub mod spell_card;
pub mod stage;

#[cfg(feature = "score-file")]
pub mod score;

pub use difficulty::Difficulty;
pub use errors::{InvalidCardId, InvalidDifficultyId, InvalidShotType, InvalidStageId};
pub use game_id::GameId;
#[cfg(feature = "score-file")]
pub use score::{PracticeRecord, ScoreFile, SpellCardRecord, SpellPracticeRecord};
pub use shot_type::ShotType;
pub use spell_card::{SpellCard, SpellCardInfo, SpellType};
pub use stage::{Stage, StageProgress};
pub struct Touhou13;
pub struct Touhou17;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Character {
    Reimu,
    Marisa,
    Sakuya,
    Sanae,
}

impl Character {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Reimu => "Reimu",
            Self::Marisa => "Marisa",
            Self::Sakuya => "Sakuya",
            Self::Sanae => "Sanae",
        }
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub trait GameValue: Debug + Copy + Sync + Send + Unpin + 'static {
    type RawValue;
    type ConversionError: Error;

    fn game_id(&self) -> GameId;
    fn raw_id(&self) -> Self::RawValue;
    fn from_raw(id: Self::RawValue, game: GameId) -> Result<Self, Self::ConversionError>;
    fn name(&self) -> &'static str;
}

pub trait Game: Sized + Sync + Send + Unpin + 'static {
    type SpellID: GameValue<RawValue = u32, ConversionError = errors::InvalidCardId>;
    type ShotTypeID: GameValue<RawValue = u16, ConversionError = errors::InvalidShotType>;
    type StageID: GameValue<RawValue = u16, ConversionError = errors::InvalidStageId>;
    type DifficultyID: GameValue<RawValue = u16, ConversionError = errors::InvalidDifficultyId>;

    fn game_id(&self) -> GameId;
    fn card_info(id: Self::SpellID) -> &'static SpellCardInfo<Self>;
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
                Some(self.cmp(other))
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

pub(super) use impl_wrapper_traits;
