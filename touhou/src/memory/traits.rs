use serde::de::DeserializeOwned;
use serde::Serialize;

use super::types::{LocationType, SpellState};
use crate::types::Game;
use crate::{Difficulty, Location, ShotPower, ShotType, SpellCard, Stage};

pub trait RunData<G: Game>: Sized {
    type StageState: StageData<G>;
    type PlayerState: PlayerData<G>;

    fn difficulty(&self) -> Difficulty<G>;
    fn player(&self) -> &Self::PlayerState;
    fn stage(&self) -> &Self::StageState;
}

pub trait StageData<G: Game>: Sized {
    type BossState: BossData<G>;

    fn stage_id(&self) -> Stage<G>;
    fn active_boss(&self) -> Option<&Self::BossState>;

    fn active_spell(&self) -> Option<SpellState<G>> {
        self.active_boss().and_then(|boss| boss.active_spell())
    }
}

pub trait PauseState {
    fn paused(&self) -> bool;
}

pub trait ECLTimeline<G: Game>: StageData<G> {
    fn ecl_time(&self) -> u32;
}

pub trait BossData<G: Game>: Sized {
    fn active_spell(&self) -> Option<SpellState<G>>;
}

pub trait BossLifebars<G: Game>: BossData<G> {
    fn remaining_lifebars(&self) -> u8;
}

pub trait PlayerData<G: Game>: Sized {
    fn shot(&self) -> ShotType<G>;
    fn power(&self) -> ShotPower<G>;
    fn lives(&self) -> u8;
    fn continues_used(&self) -> u8;
    fn score(&self) -> u64;
}

pub trait BombStock<G: Game>: PlayerData<G> + Sized {
    fn bombs(&self) -> u8;
}

pub trait MissCount<G: Game>: PlayerData<G> + Sized {
    fn total_misses(&self) -> u8;
}

pub trait BombCount<G: Game>: BombStock<G> + Sized {
    fn total_bombs(&self) -> u8;
}

pub trait ResolveLocation<G: HasLocations>: Sized {
    fn resolve_location(&self) -> Option<Location<G>>;
}

pub trait GameLocation<G: Game>:
    Sized + std::fmt::Debug + Copy + Eq + Ord + std::hash::Hash + Serialize + DeserializeOwned
{
    fn name(&self) -> &'static str;
    fn index(&self) -> u64;
    fn stage(&self) -> Stage<G>;
    fn spell(&self) -> Option<SpellCard<G>>;
    fn is_end(&self) -> bool;
    fn is_boss_start(&self) -> bool;
    fn from_spell(spell: SpellCard<G>) -> Option<Self>;
}

pub trait HasLocations: Game {
    type Location: GameLocation<Self>;
}

pub trait LocationInfo<G: HasLocations>: std::fmt::Debug + Copy + Serialize {
    fn location_type(&self) -> LocationType<G>;
    fn name(&self) -> &'static str;
    fn stage(&self) -> Stage<G>;
}
