use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::{GameLocation, Locations};
use crate::types::{Game, GameId, GameValue, SpellCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpellState<G: Game> {
    spell: G::SpellID,
    captured: bool,
}

impl<G: Game> SpellState<G> {
    pub fn new(spell: G::SpellID, captured: bool) -> Self {
        Self { spell, captured }
    }

    pub fn spell_id(&self) -> G::SpellID {
        self.spell
    }

    pub fn spell(&self) -> SpellCard<G> {
        SpellCard::new(self.spell)
    }

    pub fn captured(&self) -> bool {
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

#[derive(Debug)]
#[repr(transparent)]
pub struct Location<G: Locations>(G::Location);

impl<G: Locations> Location<G> {
    pub const fn new(location: G::Location) -> Self {
        Self(location)
    }

    pub const fn unwrap(self) -> G::Location {
        self.0
    }

    pub fn name(&self) -> &'static str {
        self.0.name()
    }

    pub fn stage(&self) -> G::StageID {
        self.0.stage()
    }

    pub fn spell(&self) -> Option<SpellCard<G>> {
        self.0.spell()
    }

    pub fn is_end(&self) -> bool {
        self.0.is_end()
    }
}

impl<G: Locations> PartialEq for Location<G> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<G: Locations> Eq for Location<G> {}

impl<G: Locations> Ord for Location<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<G: Locations> PartialOrd for Location<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl<G: Locations> Clone for Location<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Locations> Copy for Location<G> {}

impl<G: Locations> Hash for Location<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<G: Locations> AsRef<G::Location> for Location<G> {
    fn as_ref(&self) -> &G::Location {
        &self.0
    }
}

impl<G: Locations> Deref for Location<G> {
    type Target = G::Location;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedAs<G: Locations> {
    game: GameId,
    value: G::Location,
}

impl<G: Locations> serde::Serialize for Location<G> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serialized = SerializedAs {
            game: G::GAME_ID,
            value: self.0,
        };

        <SerializedAs<G> as serde::Serialize>::serialize(&serialized, serializer)
    }
}

impl<'de, G: Locations> serde::Deserialize<'de> for Location<G> {
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
