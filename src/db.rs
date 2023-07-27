use std::io::Cursor;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use futures::stream::TryStreamExt;
use sqlx::{Executor, Sqlite};
use sysinfo::{ProcessRefreshKind, System, SystemExt};
use time::OffsetDateTime;
use tokio::fs;

use crate::types::{Game, SpellCardRecord};

#[derive(Debug, Clone, Copy)]
pub struct CardAttempt<G: Game> {
    pub timestamp: OffsetDateTime,
    pub card_id: u16,
    pub shot_type: G::ShotType,
    pub captured: bool,
}

impl<G: Game> CardAttempt<G> {
    pub async fn get_card_attempts<'c, C>(
        pool: C,
        card_id: u16,
        shot_type: G::ShotType,
        max_n: usize,
    ) -> anyhow::Result<Vec<Self>>
    where
        C: Executor<'c, Database = Sqlite>,
    {
        let shot_id: u8 = shot_type.into();
        let mut query = sqlx::query!(r#"SELECT ts as "ts!: OffsetDateTime", captures, attempts FROM spellcards WHERE card_id = ? AND shot_type = ? ORDER BY ts DESC"#, card_id, shot_id).fetch(pool);

        let mut prev_row = None;
        let mut ret = Vec::new();
        while let Some(row) = query.try_next().await? {
            let cur_attempts = row.attempts;
            let cur_captures = row.captures;

            if let Some(prev_row) = prev_row.replace(row) {
                if prev_row.attempts == (cur_attempts + 1) {
                    ret.push(CardAttempt {
                        timestamp: prev_row.ts,
                        captured: prev_row.captures == (cur_captures + 1),
                        shot_type,
                        card_id,
                    });

                    if ret.len() >= max_n {
                        break;
                    }
                }
            }
        }

        Ok(ret)
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

pub struct SnapshotStream<G: Game> {
    score_path: PathBuf,
    last_modified: SystemTime,
    game: PhantomData<G>,
}

impl<G: Game> SnapshotStream<G> {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let mut system = System::new();
        loop {
            system.refresh_processes_specifics(ProcessRefreshKind::new());

            if let Some(score_path) = G::find_score_file(&system) {
                let last_modified = fs::metadata(&score_path).await?.modified()?;
                return Ok(Self {
                    score_path,
                    last_modified,
                    game: PhantomData,
                });
            }
        }
    }

    pub fn score_path(&self) -> &Path {
        &self.score_path
    }

    pub async fn read_card_data(&self) -> Result<Vec<CardSnapshot<G>>, anyhow::Error> {
        let data = Cursor::new(fs::read(&self.score_path).await?);
        let timestamp = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

        let mut ret = Vec::new();
        for record in G::load_card_records(data)? {
            let record = record?;
            ret.extend(CardSnapshot::from_score_data(timestamp, &record).filter(|r| r.attempts > 0))
        }
        Ok(ret)
    }

    pub async fn refresh_snapshots(
        &mut self,
    ) -> Result<Option<(SystemTime, Vec<CardSnapshot<G>>)>, anyhow::Error> {
        let cur_time = SystemTime::now();
        let mtime = fs::metadata(&self.score_path).await?.modified()?;
        let prev_mtime = std::mem::replace(&mut self.last_modified, mtime);

        if (mtime > prev_mtime)
            && cur_time
                .duration_since(mtime)
                .map(|d| d >= Duration::from_secs(2))
                .unwrap_or(false)
        {
            self.read_card_data().await.map(|r| Some((mtime, r)))
        } else {
            Ok(None)
        }
    }
}
