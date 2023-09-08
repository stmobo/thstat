use std::fmt::Debug;

use super::{Difficulty, Game, ShotType, SpellCard, Stage};

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

pub trait PracticeRecord<G: Game>: Sized + Debug {
    fn high_score(&self) -> u32;
    fn attempts(&self) -> u32;
    fn shot_type(&self) -> ShotType<G>;
    fn difficulty(&self) -> Difficulty<G>;
    fn stage(&self) -> Stage<G>;
}

pub trait ScoreFile<G: Game>: Sized + Debug {
    type SpellCardRecord: SpellCardRecord<G>;
    type PracticeRecord: PracticeRecord<G>;

    fn spell_cards(&self) -> &[Self::SpellCardRecord];
    fn practice_records(&self) -> &[Self::PracticeRecord];
}
