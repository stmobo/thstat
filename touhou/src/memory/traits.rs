use super::types::SpellState;
use crate::types::Game;

pub trait RunData<G: Game>: Sized {
    type StageState: StageData<G>;
    type PlayerState: PlayerData<G>;

    fn difficulty(&self) -> G::DifficultyID;
    fn player(&self) -> &Self::PlayerState;
    fn stage(&self) -> &Self::StageState;
}

pub trait StageData<G: Game>: Sized {
    type BossState: BossData<G>;

    fn stage_id(&self) -> G::StageID;
    fn ecl_time(&self) -> u32;
    fn active_boss(&self) -> Option<&Self::BossState>;

    fn active_spell(&self) -> Option<SpellState<G>> {
        self.active_boss().and_then(|boss| boss.active_spell())
    }
}

pub trait BossData<G: Game>: Sized {
    fn active_spell(&self) -> Option<SpellState<G>>;
}

pub trait BossLifebars {
    fn remaining_lifebars(&self) -> u8;
}

pub trait PlayerData<G: Game>: Sized {
    fn shot(&self) -> G::ShotTypeID;
    fn lives(&self) -> u8;
    fn continues_used(&self) -> u8;
    fn score(&self) -> u64;
}

pub trait BombStock: Sized {
    fn bombs(&self) -> u8;
}

pub trait MissCount: Sized {
    fn total_misses(&self) -> u8;
}

pub trait BombCount: Sized {
    fn total_bombs(&self) -> u8;
}
