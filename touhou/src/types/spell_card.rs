//! Types for working with spell card information.

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;
use std::str;

use serde::{Deserialize, Serialize};

use super::{impl_wrapper_traits, Difficulty, Game, GameValue, Stage};

/// Contains information for specific spell cards.
///
/// These can be accessed via [`SpellCard`] instances, or via game-specific `SpellId` instances (which automatically deref to this type).
#[derive(Debug, Serialize, Deserialize)]
pub struct SpellCardInfo<G: Game> {
    /// The translated English name of this spell card.
    pub name: &'static str,
    /// The difficulty level in which this spell card appears.
    pub difficulty: Difficulty<G>,
    /// The stage in which this spell card appears.
    pub stage: Stage<G>,
    /// Where this spell card appears (i.e. as a midboss spell, a boss spell, a Last Spell, or a Last Word).
    pub spell_type: SpellType,
    /// When this spell card appears in its associated boss fight.
    pub sequence_number: u32,
}

impl<G: Game> Clone for SpellCardInfo<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Game> Copy for SpellCardInfo<G> {}

/// Represents the different places in which a spell card can appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellType {
    Midboss,
    Boss,
    LastSpell,
    LastWord,
}

impl SpellType {
    /// Does this spell card appear during a game stage?
    ///
    /// This returns `true` for all cards except Last Words.
    pub fn is_stage(&self) -> bool {
        !matches!(self, Self::LastWord)
    }

    /// Does this spell card appear during a boss fight?
    ///
    /// This returns `true` for boss cards and Last Spells, but not midboss cards and Last Words.
    pub fn is_boss(&self) -> bool {
        matches!(self, Self::Boss) || matches!(self, Self::LastSpell)
    }
}

/// Represents a spell card from one of the Touhou games.
///
/// This is a convenience wrapper around the game-specific `SpellId` types defined elsewhere in this crate.
/// To access the inner type, use [`Self::unwrap`].
#[repr(transparent)]
pub struct SpellCard<G: Game>(G::SpellID);

impl<G: Game> SpellCard<G> {
    /// Wraps a game-specific spell ID type in a new wrapper instance.
    pub const fn new(card_id: G::SpellID) -> Self {
        Self(card_id)
    }

    /// Gets the inner game-specific type from this instance.
    pub const fn unwrap(self) -> G::SpellID {
        self.0
    }

    /// Gets the ID number of this spell card directly as a `u32`.
    pub fn id(&self) -> u32 {
        self.0.raw_id()
    }

    /// Gets a reference to the static information for this card.
    pub fn info(&self) -> &'static SpellCardInfo<G> {
        G::card_info(self.0)
    }

    /// Gets the name of this spell card.
    pub fn name(&self) -> &'static str {
        self.info().name
    }

    /// Gets the difficulty level of this spell card.
    pub fn difficulty(&self) -> Difficulty<G> {
        self.info().difficulty
    }

    /// Gets the stage associated with this spell card.
    pub fn stage(&self) -> Stage<G> {
        self.info().stage
    }
}

impl<G: Game> AsRef<G::SpellID> for SpellCard<G> {
    fn as_ref(&self) -> &G::SpellID {
        &self.0
    }
}

impl<G: Game> Deref for SpellCard<G> {
    type Target = G::SpellID;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G: Game> Debug for SpellCard<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpellCard<{}>({:?} : {})",
            self.0.game_id().abbreviation(),
            self.0,
            self.name()
        )
    }
}

impl<G: Game> Display for SpellCard<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} #{}: {}",
            self.0.game_id().abbreviation(),
            self.0.raw_id(),
            self.name()
        )
    }
}

impl_wrapper_traits!(SpellCard, u32, G::SpellID, IterAll);
