use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Executor, FromRow, Row, Sqlite};
use time::OffsetDateTime;

use crate::types::{
    Difficulty, Game, GameId, PracticeRecord, ShotType, ShotTypeId, SpellCard, SpellCardId,
    SpellCardInfo, SpellCardRecord, Stage,
};

pub type CardSnapshotKey<G> = (SpellCard<G>, ShotType<G>);
pub type PracticeSnapshotKey<G> = (Difficulty, ShotType<G>, Stage);

#[derive(Debug, Serialize, Deserialize)]
pub struct CardSnapshot<G: Game> {
    pub timestamp: OffsetDateTime,
    pub card: SpellCard<G>,
    pub shot_type: ShotType<G>,
    pub captures: u32,
    pub attempts: u32,
    pub max_bonus: u32,
}

impl<G: Game> CardSnapshot<G> {
    pub(super) fn key(&self) -> CardSnapshotKey<G> {
        (self.card, self.shot_type)
    }

    pub fn card_id(&self) -> u32 {
        self.card.id()
    }

    pub fn card_info(&self) -> &'static SpellCardInfo {
        self.card.info()
    }

    pub fn difficulty(&self) -> Difficulty {
        self.card_info().difficulty
    }

    pub fn stage(&self) -> Stage {
        self.card_info().stage
    }

    pub fn card_name(&self) -> &'static str {
        self.card_info().name
    }

    pub async fn insert<'c, C>(&self, executor: C) -> Result<SqliteQueryResult, sqlx::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let raw_card_id = self.card.raw_id();
        let raw_shot_id = self.shot_type.raw_id();
        let game_id = self.card.game_id();

        assert_eq!(self.card.game_id(), self.shot_type.game_id());

        sqlx::query!(
            r#"
            INSERT INTO spellcards (ts, card_id, shot_type, game, captures, attempts, max_bonus)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            self.timestamp,
            raw_card_id,
            raw_shot_id,
            game_id,
            self.captures,
            self.attempts,
            self.max_bonus
        )
        .execute(executor)
        .await
    }

    pub async fn get_first_snapshot_after<'c, C>(
        pool: C,
        card: SpellCard<G>,
        shot_type: ShotType<G>,
        after_time: OffsetDateTime,
    ) -> Result<Option<Self>, anyhow::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let raw_card_id = card.raw_id();
        let raw_shot_id = shot_type.raw_id();

        sqlx::query_as::<_, Self>(
            "SELECT * FROM spellcards WHERE card_id = ? AND shot_type = ? AND ts >= ? ORDER BY ts ASC LIMIT 1",
        )
        .bind(raw_card_id)
        .bind(raw_shot_id)
        .bind(after_time)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.into())
    }

    pub async fn get_last_snapshot<'c, C>(
        pool: C,
        card: SpellCard<G>,
        shot_type: ShotType<G>,
    ) -> Result<Option<Self>, anyhow::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let raw_card_id = card.raw_id();
        let raw_shot_id = shot_type.raw_id();

        sqlx::query_as::<_, Self>(
            "SELECT * FROM spellcards WHERE card_id = ? AND shot_type = ? ORDER BY ts DESC",
        )
        .bind(raw_card_id)
        .bind(raw_shot_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.into())
    }

    pub fn from_score_data(
        timestamp: OffsetDateTime,
        data: &G::SpellCardRecord,
    ) -> impl Iterator<Item = Self> + '_ {
        data.shot_types().iter().map(move |k| CardSnapshot {
            timestamp,
            card: data.card(),
            captures: data.captures(k),
            attempts: data.attempts(k),
            max_bonus: data.max_bonus(k),
            shot_type: *k,
        })
    }
}

impl<G: Game> FromRow<'_, SqliteRow> for CardSnapshot<G> {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let game_id: GameId = row.try_get("game")?;
        let card_id = G::SpellID::from_raw(row.try_get("card_id")?, game_id)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let shot_id = G::ShotTypeID::from_raw(row.try_get("shot_type")?, game_id)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Self {
            card: SpellCard::new(card_id),
            shot_type: ShotType::new(shot_id),
            timestamp: row.try_get("ts")?,
            captures: row.try_get("captures")?,
            attempts: row.try_get("attempts")?,
            max_bonus: row.try_get("max_bonus")?,
        })
    }
}

impl<G: Game> Clone for CardSnapshot<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Game> Copy for CardSnapshot<G> {}

#[derive(Debug, Serialize, Deserialize)]
pub struct PracticeSnapshot<G: Game> {
    pub timestamp: OffsetDateTime,
    pub difficulty: Difficulty,
    pub shot_type: ShotType<G>,
    pub stage: Stage,
    pub attempts: u32,
    pub high_score: u32,
}

impl<G: Game> PracticeSnapshot<G> {
    pub(super) fn key(&self) -> PracticeSnapshotKey<G> {
        (self.difficulty, self.shot_type, self.stage)
    }

    pub async fn insert<'c, C>(&self, executor: C) -> Result<SqliteQueryResult, sqlx::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let raw_shot_type = self.shot_type.raw_id();
        let game_id = self.shot_type.game_id();

        sqlx::query!(
            r#"
            INSERT INTO practices (ts, difficulty, shot_type, game, stage, attempts, high_score)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            self.timestamp,
            self.difficulty,
            raw_shot_type,
            game_id,
            self.stage,
            self.attempts,
            self.high_score
        )
        .execute(executor)
        .await
    }

    pub async fn get_last_snapshot<'c, C>(
        pool: C,
        difficulty: Difficulty,
        shot_type: ShotType<G>,
        stage: Stage,
    ) -> Result<Option<Self>, anyhow::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM practices WHERE difficulty = ? AND shot_type = ? AND stage = ? ORDER BY ts DESC",
        )
        .bind(difficulty)
        .bind(shot_type.raw_id())
        .bind(stage)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.into())
    }

    pub fn from_score_data(timestamp: OffsetDateTime, data: &G::PracticeRecord) -> Self {
        Self {
            timestamp,
            difficulty: data.difficulty(),
            shot_type: data.shot_type(),
            stage: data.stage(),
            attempts: data.attempts(),
            high_score: data.high_score(),
        }
    }
}

impl<G: Game> FromRow<'_, SqliteRow> for PracticeSnapshot<G> {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let game_id: GameId = row.try_get("game")?;
        let shot_id = G::ShotTypeID::from_raw(row.try_get("shot_type")?, game_id)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Self {
            shot_type: ShotType::new(shot_id),
            timestamp: row.try_get("ts")?,
            difficulty: row.try_get("difficulty")?,
            stage: row.try_get("stage")?,
            attempts: row.try_get("attempts")?,
            high_score: row.try_get("high_score")?,
        })
    }
}

impl<G: Game> Clone for PracticeSnapshot<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: Game> Copy for PracticeSnapshot<G> {}
