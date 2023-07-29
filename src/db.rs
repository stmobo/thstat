use std::collections::HashMap;
use std::hash::Hash;
use std::io::Cursor;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::bail;
use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Acquire, Executor, Sqlite};
use sysinfo::{ProcessRefreshKind, System, SystemExt};
use thiserror::Error;
use time::OffsetDateTime;
use tokio::fs;

use crate::types::{
    Difficulty, Game, IterableEnum, PracticeRecord, ScoreFile, SpellCardInfo, SpellCardRecord,
    Stage,
};

#[derive(Debug, Clone, Copy)]
struct CardSnapshotKey<G: Game>(u16, G::ShotType);

impl<G: Game> PartialEq for CardSnapshotKey<G> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0) && (self.1 == other.1)
    }
}

impl<G: Game> Eq for CardSnapshotKey<G> {}

impl<G: Game> Hash for CardSnapshotKey<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
struct PracticeSnapshotKey<G: Game>(Difficulty, G::ShotType, Stage);

impl<G: Game> PartialEq for PracticeSnapshotKey<G> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0) && (self.1 == other.1) && (self.2 == other.2)
    }
}

impl<G: Game> Eq for PracticeSnapshotKey<G> {}

impl<G: Game> Hash for PracticeSnapshotKey<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct CardSnapshot<G: Game> {
    #[sqlx(rename = "ts")]
    pub timestamp: OffsetDateTime,
    pub card_id: u16,
    #[sqlx(try_from = "u8")]
    pub shot_type: G::ShotType,
    pub captures: u32,
    pub attempts: u32,
    pub max_bonus: u32,
}

impl<G: Game> CardSnapshot<G> {
    fn key(&self) -> CardSnapshotKey<G> {
        CardSnapshotKey(self.card_id, self.shot_type)
    }

    pub fn card_info(&self) -> &'static SpellCardInfo {
        G::get_card_info(self.card_id - 1).unwrap()
    }

    pub fn difficulty(&self) -> Difficulty {
        self.card_info().difficulty()
    }

    pub fn stage(&self) -> Stage {
        self.card_info().stage()
    }

    pub fn card_name(&self) -> &'static str {
        self.card_info().name()
    }

    pub async fn insert<'c, C>(&self, executor: C) -> Result<SqliteQueryResult, sqlx::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let shot_type: u8 = self.shot_type.into();
        sqlx::query!(
            r#"
            INSERT INTO spellcards (ts, card_id, shot_type, captures, attempts, max_bonus)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            self.timestamp,
            self.card_id,
            shot_type,
            self.captures,
            self.attempts,
            self.max_bonus
        )
        .execute(executor)
        .await
    }

    pub async fn get_last_snapshot<'c, C>(
        pool: C,
        card_id: u16,
        shot_type: G::ShotType,
    ) -> Result<Option<Self>, anyhow::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let shot_type: u8 = shot_type.into();
        sqlx::query_as::<_, Self>(
            "SELECT * FROM spellcards WHERE card_id = ? AND shot_type = ? ORDER BY ts DESC",
        )
        .bind(card_id)
        .bind(shot_type)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.into())
    }

    pub fn from_score_data(
        timestamp: OffsetDateTime,
        data: &G::SpellCardRecord,
    ) -> impl Iterator<Item = Self> + '_ {
        G::shot_types().map(move |k| CardSnapshot {
            timestamp,
            card_id: data.card_id(),
            captures: data.captures(&k),
            attempts: data.attempts(&k),
            max_bonus: data.max_bonus(&k),
            shot_type: k,
        })
    }
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct PracticeSnapshot<G: Game> {
    #[sqlx(rename = "ts")]
    pub timestamp: OffsetDateTime,
    #[sqlx(try_from = "u8")]
    pub difficulty: Difficulty,
    #[sqlx(try_from = "u8")]
    pub shot_type: G::ShotType,
    #[sqlx(try_from = "u8")]
    pub stage: Stage,
    pub attempts: u32,
    pub high_score: u32,
}

impl<G: Game> PracticeSnapshot<G> {
    fn key(&self) -> PracticeSnapshotKey<G> {
        PracticeSnapshotKey(self.difficulty, self.shot_type, self.stage)
    }

    pub async fn insert<'c, C>(&self, executor: C) -> Result<SqliteQueryResult, sqlx::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let shot_type: u8 = self.shot_type.into();
        let difficulty: u8 = self.difficulty.into();
        let stage: u8 = self.stage.into();

        sqlx::query!(
            r#"
            INSERT INTO practices (ts, difficulty, shot_type, stage, attempts, high_score)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            self.timestamp,
            difficulty,
            shot_type,
            stage,
            self.attempts,
            self.high_score
        )
        .execute(executor)
        .await
    }

    pub async fn get_last_snapshot<'c, C>(
        pool: C,
        difficulty: Difficulty,
        shot_type: G::ShotType,
        stage: Stage,
    ) -> Result<Option<Self>, anyhow::Error>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let shot_type: u8 = shot_type.into();
        let difficulty: u8 = difficulty.into();
        let stage: u8 = stage.into();

        sqlx::query_as::<_, Self>(
            "SELECT * FROM practices WHERE difficulty = ? AND shot_type = ? AND stage = ? ORDER BY ts DESC",
        )
        .bind(difficulty)
        .bind(shot_type)
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

#[derive(Debug, Clone, Copy)]
pub struct CardAttemptInfo {
    is_capture: bool,
    total_attempts: u32,
    total_captures: u32,
}

impl CardAttemptInfo {
    pub fn is_capture(&self) -> bool {
        self.is_capture
    }

    pub fn total_attempts(&self) -> u32 {
        self.total_attempts
    }

    pub fn total_captures(&self) -> u32 {
        self.total_captures
    }
}

#[derive(Debug, Clone)]
pub struct UpdateEvent<G: Game> {
    timestamp: OffsetDateTime,
    shot_type: G::ShotType,
    difficulty: Difficulty,
    stage: Stage,
    practice_no: Option<u32>,
    attempted_cards: HashMap<u16, CardAttemptInfo>,
}

impl<G: Game> UpdateEvent<G> {
    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn shot_type(&self) -> G::ShotType {
        self.shot_type
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn is_practice(&self) -> bool {
        self.practice_no.is_some()
    }

    pub fn practice_no(&self) -> Option<u32> {
        self.practice_no
    }

    pub fn n_attempted_cards(&self) -> usize {
        self.attempted_cards.len()
    }

    pub fn attempted_cards(&self) -> impl Iterator<Item = (u16, &CardAttemptInfo)> + '_ {
        self.attempted_cards.iter().map(|kv| (*kv.0, kv.1))
    }
}

#[derive(Debug, Clone)]
pub struct FileSnapshot<G: Game> {
    timestamp: OffsetDateTime,
    cards: HashMap<CardSnapshotKey<G>, CardSnapshot<G>>,
    practices: HashMap<PracticeSnapshotKey<G>, PracticeSnapshot<G>>,
}

impl<G: Game> FileSnapshot<G> {
    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn get_card(&self, shot_type: G::ShotType, card_id: u16) -> Option<&CardSnapshot<G>> {
        self.cards.get(&CardSnapshotKey(card_id, shot_type))
    }

    pub fn iter_cards(&self) -> impl Iterator<Item = &CardSnapshot<G>> + '_ {
        self.cards.values()
    }

    pub fn get_practice(
        &self,
        difficulty: Difficulty,
        shot_type: G::ShotType,
        stage: Stage,
    ) -> Option<&PracticeSnapshot<G>> {
        self.practices
            .get(&PracticeSnapshotKey(difficulty, shot_type, stage))
    }

    pub fn iter_practices(&self) -> impl Iterator<Item = &PracticeSnapshot<G>> + '_ {
        self.practices.values()
    }

    pub async fn insert<'c, C>(&self, conn: C) -> Result<(), sqlx::Error>
    where
        C: Acquire<'c, Database = Sqlite>,
    {
        let mut tx = conn.begin().await?;

        for practice in self.practices.values() {
            practice.insert(&mut tx).await?;
        }

        for card in self.cards.values() {
            card.insert(&mut tx).await?;
        }

        tx.commit().await
    }

    pub fn get_updates(&self, other: &FileSnapshot<G>) -> Vec<UpdateEvent<G>> {
        if self.timestamp > other.timestamp {
            return other.get_updates(self);
        }

        let prev_card_attempts: HashMap<CardSnapshotKey<G>, (u32, u32)> = self
            .cards
            .iter()
            .map(|(k, v)| (*k, (v.attempts, v.captures)))
            .collect();

        let mut grouped_card_attempts: HashMap<
            PracticeSnapshotKey<G>,
            (u32, u32, HashMap<u16, CardAttemptInfo>),
        > = HashMap::new();

        for (key, new_card) in other.cards.iter() {
            let (prev_attempts, prev_captures) = prev_card_attempts
                .get(key)
                .map(|p| (p.0, p.1))
                .unwrap_or((0, 0));

            if new_card.attempts == (prev_attempts + 1) {
                let prac_key = PracticeSnapshotKey(
                    new_card.difficulty(),
                    new_card.shot_type,
                    new_card.stage(),
                );

                let attempt_info = CardAttemptInfo {
                    is_capture: (new_card.captures == (prev_captures + 1)),
                    total_attempts: new_card.attempts,
                    total_captures: new_card.captures,
                };

                grouped_card_attempts
                    .entry(prac_key)
                    .or_default()
                    .2
                    .insert(new_card.card_id, attempt_info);
            }
        }

        for (key, snapshot) in &self.practices {
            grouped_card_attempts.entry(*key).or_default().0 = snapshot.attempts;
        }

        for (key, snapshot) in &other.practices {
            grouped_card_attempts.entry(*key).or_default().1 = snapshot.attempts;
        }

        grouped_card_attempts
            .into_iter()
            .filter_map(|(key, group)| {
                let (prev_practices, new_practices, attempted_cards) = group;
                let PracticeSnapshotKey(difficulty, shot_type, stage) = key;

                if !attempted_cards.is_empty() {
                    let practice_no = if new_practices == (prev_practices + 1) {
                        Some(new_practices)
                    } else {
                        None
                    };

                    Some(UpdateEvent {
                        timestamp: other.timestamp,
                        practice_no,
                        shot_type,
                        difficulty,
                        stage,
                        attempted_cards,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct SnapshotStream<G: Game> {
    system: System,
    score_path: PathBuf,
    last_modified: SystemTime,
    phantom: PhantomData<G>,
}

impl<G: Game> SnapshotStream<G> {
    pub async fn new() -> Result<Self, anyhow::Error> {
        eprint!("Waiting for Touhou... ");
        let mut system = System::new();
        loop {
            system.refresh_processes_specifics(ProcessRefreshKind::new());

            if let Some(score_path) = G::find_score_file(&system) {
                eprintln!("found score file at {}", score_path.display());

                let last_modified = fs::metadata(&score_path).await?.modified()?;
                return Ok(Self {
                    system,
                    score_path,
                    last_modified,
                    phantom: PhantomData,
                });
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn score_path(&self) -> &Path {
        &self.score_path
    }

    pub async fn read_snapshot_data(&mut self) -> Result<FileSnapshot<G>, anyhow::Error> {
        let data = Cursor::new(fs::read(&self.score_path).await?);
        let timestamp = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let score_data = G::load_score_file(data)?;

        let cards = score_data
            .spell_cards()
            .iter()
            .flat_map(|data| {
                CardSnapshot::from_score_data(timestamp, data).filter(|r| r.attempts > 0)
            })
            .map(|c| (c.key(), c))
            .collect();

        let practices = score_data
            .practice_records()
            .iter()
            .map(|data| PracticeSnapshot::from_score_data(timestamp, data))
            .map(|c| (c.key(), c))
            .collect();

        Ok(FileSnapshot {
            timestamp,
            cards,
            practices,
        })
    }

    pub async fn refresh_snapshots(
        &mut self,
    ) -> Result<Option<(SystemTime, FileSnapshot<G>)>, anyhow::Error> {
        let cur_time = SystemTime::now();
        let mtime = fs::metadata(&self.score_path).await?.modified()?;

        if (mtime > self.last_modified)
            && cur_time
                .duration_since(mtime)
                .map(|d| d >= Duration::from_secs(2))
                .unwrap_or(false)
        {
            self.last_modified = mtime;
            self.read_snapshot_data().await.map(|r| Some((mtime, r)))
        } else {
            Ok(None)
        }
    }
}
