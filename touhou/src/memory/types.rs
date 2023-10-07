/// Common wrappers and containers for game state extracted from running Touhou games.
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::{Deref, RangeInclusive};

use serde::{Deserialize, Serialize};

use super::{GameLocation, HasLocations};
use crate::types::errors::InvalidStageId;
use crate::types::{Game, GameId, GameValue, SpellCard, Stage};

/// The status of a spell in a running game.
///
/// This type ties together a spell card as well as a flag
/// indicating whether or not the spell has been (or is on track to be) captured.
///
/// Values of this type are returned from [`BossData::active_spell`](super::traits::BossData::active_spell).
///
/// This type derefs to the underlying [`G::SpellID`](Game::SpellID) type, which in turn should deref
/// to the given spell's [`SpellCardInfo`](crate::types::SpellCardInfo) structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellState<G: Game> {
    spell: G::SpellID,
    captured: bool,
}

impl<G: Game> SpellState<G> {
    pub const fn new(spell: G::SpellID, captured: bool) -> Self {
        Self { spell, captured }
    }

    pub const fn spell_id(&self) -> G::SpellID {
        self.spell
    }

    pub const fn spell(&self) -> SpellCard<G> {
        SpellCard::new(self.spell)
    }

    pub const fn captured(&self) -> bool {
        self.captured
    }

    pub(crate) fn raw_spell_id(&self) -> u32 {
        self.spell.raw_id()
    }
}

impl<G: Game> Deref for SpellState<G> {
    type Target = G::SpellID;

    fn deref(&self) -> &Self::Target {
        &self.spell
    }
}

/// Represents a gameplay location within a running Touhou game.
///
/// What exactly counts as a 'location' heavily depends on the game
/// and what memory-reading capabilities have been implemented for it:
/// - [PCB](`crate::th07::memory`) and [IN](`crate::th08::memory`) have the most fine-grained location definitions (bugs aside),
/// covering nonspell attacks, spell cards, and sections within stages; the stage sections generally match up with thprac's stage/section warp functionality.
/// - The memory reader for [MoF](`crate::th10::memory`) currently lacks the ability to distinguish sections within stages; nonetheless,
/// the reader can fully resolve both nonspell and spell card attacks for both midbosses and stage bosses.
#[derive(Debug)]
#[repr(transparent)]
pub struct Location<G: HasLocations>(G::Location);

impl<G: HasLocations> Location<G> {
    pub const fn new(location: G::Location) -> Self {
        Self(location)
    }

    pub const fn unwrap(self) -> G::Location {
        self.0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn index(&self) -> u64 {
        self.0.index()
    }

    pub fn stage(&self) -> Stage<G> {
        self.0.stage()
    }

    pub fn spell(&self) -> Option<SpellCard<G>> {
        self.0.spell()
    }

    pub fn is_end(&self) -> bool {
        self.0.is_end()
    }

    pub fn is_boss_start(&self) -> bool {
        self.0.is_boss_start()
    }

    pub fn from_spell(spell: SpellCard<G>) -> Option<Self> {
        G::Location::from_spell(spell).map(Self)
    }
}

impl<G: HasLocations> PartialEq for Location<G> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<G: HasLocations> Eq for Location<G> {}

impl<G: HasLocations> Ord for Location<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<G: HasLocations> PartialOrd for Location<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<G: HasLocations> Clone for Location<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: HasLocations> Copy for Location<G> {}

impl<G: HasLocations> Hash for Location<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<G: HasLocations> AsRef<G::Location> for Location<G> {
    fn as_ref(&self) -> &G::Location {
        &self.0
    }
}

impl<G: HasLocations> Deref for Location<G> {
    type Target = G::Location;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedAs<G: HasLocations> {
    game: GameId,
    value: G::Location,
}

impl<G: HasLocations> serde::Serialize for Location<G> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serialized = SerializedAs {
            game: G::GAME_ID,
            value: self.0,
        };

        <SerializedAs<G> as serde::Serialize>::serialize(&serialized, serializer)
    }
}

impl<'de, G: HasLocations> serde::Deserialize<'de> for Location<G> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let deserialized: SerializedAs<G> =
            <SerializedAs<G> as serde::Deserialize<'de>>::deserialize(deserializer)?;

        if deserialized.game == G::GAME_ID {
            Ok(Self(deserialized.value))
        } else {
            Err(<D::Error as serde::de::Error>::custom(format!(
                "Invalid game ID (expected {}, got {})",
                G::GAME_ID,
                deserialized.game
            )))
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidLocationData {
    IncorrectGame {
        game: GameId,
        expected: GameId,
    },
    InvalidIndex {
        game: GameId,
        stage: &'static str,
        index: u64,
        valid: RangeInclusive<u64>,
    },
    MissingSpell {
        game: GameId,
        stage: &'static str,
        loc_name: &'static str,
        valid: RangeInclusive<u32>,
    },
    InvalidSpell {
        game: GameId,
        stage: &'static str,
        loc_name: &'static str,
        valid: RangeInclusive<u32>,
    },
    InvalidStage(InvalidStageId),
    NoStageData {
        game: GameId,
        stage: &'static str,
    },
}

impl Error for InvalidLocationData {}

impl Display for InvalidLocationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoStageData { game, stage } => write!(
                f,
                "No stage data defined for {} stage {}",
                game.abbreviation(),
                stage
            ),
            Self::IncorrectGame { game, expected } => write!(
                f,
                "Incorrect game ID {} found (expected {})",
                game.abbreviation(),
                expected.abbreviation()
            ),
            Self::InvalidIndex {
                game,
                stage,
                index,
                valid,
            } => write!(
                f,
                "Invalid location index {} for {} stage {} (valid indices are {:?})",
                index,
                game.abbreviation(),
                stage,
                valid
            ),
            Self::MissingSpell {
                game,
                stage,
                loc_name,
                valid,
            } => {
                write!(
                    f,
                    "No spell ID provided for {} stage {} {} (valid spell IDs are {:?})",
                    game.abbreviation(),
                    stage,
                    loc_name,
                    valid
                )
            }
            Self::InvalidSpell {
                game,
                stage,
                loc_name,
                valid,
            } => write!(
                f,
                "Invalid spell ID provided for {} stage {} {} (valid spell IDs are {:?})",
                game.abbreviation(),
                stage,
                loc_name,
                valid
            ),
            Self::InvalidStage(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AnyLocation {
    game: GameId,
    stage: u16,
    index: u64,
    spell: Option<u32>,
}

impl AnyLocation {
    pub(crate) const fn new(game: GameId, stage: u16, index: u64, spell: Option<u32>) -> Self {
        Self {
            game,
            stage,
            index,
            spell,
        }
    }

    pub fn game(&self) -> GameId {
        self.game
    }

    pub fn downcast<G>(self) -> Result<Location<G>, InvalidLocationData>
    where
        G: HasLocations,
        Location<G>: TryFrom<Self, Error = InvalidLocationData>,
    {
        self.try_into()
    }

    pub fn downcast_stage<G: HasLocations>(&self) -> Result<G::StageID, InvalidStageId> {
        <G::StageID as GameValue>::from_raw(self.stage, self.game)
    }

    pub(crate) fn stage(&self) -> u16 {
        self.stage
    }

    pub(crate) fn index(&self) -> u64 {
        self.index
    }

    pub(crate) fn spell(&self) -> Option<u32> {
        self.spell
    }
}

impl Display for AnyLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::types::VisitGame;

        struct Visitor<'a, 'b>(AnyLocation, &'a mut std::fmt::Formatter<'b>);

        impl<'a, 'b> VisitGame for Visitor<'a, 'b> {
            type Output = std::fmt::Result;

            #[cfg(feature = "th07")]
            fn visit_th07(self) -> Self::Output {
                use crate::th07::Location;
                Location::try_from(self.0).unwrap().fmt(self.1)
            }

            #[cfg(feature = "th08")]
            fn visit_th08(self) -> Self::Output {
                use crate::th08::Location;
                Location::try_from(self.0).unwrap().fmt(self.1)
            }
        }

        Visitor(*self, f).accept_id(self.game)
    }
}
