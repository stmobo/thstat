use std::ops::Deref;

use crate::types::{Game, GameValue, SpellCard};

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
