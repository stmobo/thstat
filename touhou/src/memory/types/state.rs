use std::ops::Deref;

use crate::types::{Game, GameValue, SpellCard};

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
