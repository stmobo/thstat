use std::collections::HashMap;


use std::io::{Cursor, Read};
use std::marker::PhantomData;
use std::path::{Path};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Sqlite};
use sysinfo::{SystemExt};
use time::OffsetDateTime;
use tokio::fs;

use crate::types::{
    Difficulty, Game, PracticeRecord, ScoreFile, ShotType, SpellCard, Stage,
};

mod row_types;

pub use row_types::{CardSnapshot, CardSnapshotKey, PracticeSnapshot, PracticeSnapshotKey};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvent<G: Game> {
    timestamp: OffsetDateTime,
    shot_type: ShotType<G>,
    difficulty: Difficulty,
    stage: Stage,
    practice_no: Option<u32>,
    attempted_cards: HashMap<SpellCard<G>, CardAttemptInfo>,
}

impl<G: Game> UpdateEvent<G> {
    fn cmp_key(&self) -> (OffsetDateTime, PracticeSnapshotKey<G>) {
        (
            self.timestamp,
            (self.difficulty, self.shot_type, self.stage),
        )
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn shot_type(&self) -> ShotType<G> {
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

    pub fn attempted_cards(&self) -> impl Iterator<Item = (SpellCard<G>, &CardAttemptInfo)> + '_ {
        let mut tmp: Vec<_> = self.attempted_cards.iter().collect();
        tmp.sort_unstable_by_key(|kv| kv.0);
        tmp.into_iter().map(|kv| (*kv.0, kv.1))
    }
}

impl<G: Game> PartialEq for UpdateEvent<G> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_key() == other.cmp_key()
    }
}

impl<G: Game> Eq for UpdateEvent<G> {}

impl<G: Game> PartialOrd for UpdateEvent<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp_key().partial_cmp(&other.cmp_key())
    }
}

impl<G: Game> Ord for UpdateEvent<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_key().cmp(&other.cmp_key())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot<G: Game> {
    timestamp: OffsetDateTime,
    cards: HashMap<CardSnapshotKey<G>, CardSnapshot<G>>,
    practices: HashMap<PracticeSnapshotKey<G>, PracticeSnapshot<G>>,
}

impl<G: Game> FileSnapshot<G> {
    pub fn new<R: Read>(
        game: &G,
        timestamp: OffsetDateTime,
        src: R,
    ) -> Result<Self, anyhow::Error> {
        let score_data = game.load_score_file(src)?;

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

        Ok(Self {
            timestamp,
            cards,
            practices,
        })
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn get_card(&self, shot_type: ShotType<G>, card: SpellCard<G>) -> Option<&CardSnapshot<G>> {
        self.cards.get(&(card, shot_type))
    }

    pub fn iter_cards(&self) -> impl Iterator<Item = &CardSnapshot<G>> + '_ {
        let mut tmp: Vec<_> = self.cards.values().collect();
        tmp.sort_unstable_by_key(|snapshot| snapshot.key());
        tmp.into_iter()
    }

    pub fn get_practice(
        &self,
        difficulty: Difficulty,
        shot_type: ShotType<G>,
        stage: Stage,
    ) -> Option<&PracticeSnapshot<G>> {
        self.practices.get(&(difficulty, shot_type, stage))
    }

    pub fn iter_practices(&self) -> impl Iterator<Item = &PracticeSnapshot<G>> + '_ {
        let mut tmp: Vec<_> = self.practices.values().collect();
        tmp.sort_unstable_by_key(|snapshot| snapshot.key());
        tmp.into_iter()
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
            (u32, u32, HashMap<SpellCard<G>, CardAttemptInfo>),
        > = HashMap::new();

        for (key, new_card) in other.cards.iter() {
            let (prev_attempts, prev_captures) = prev_card_attempts
                .get(key)
                .map(|p| (p.0, p.1))
                .unwrap_or((0, 0));

            if new_card.attempts == (prev_attempts + 1) {
                let prac_key = (new_card.difficulty(), new_card.shot_type, new_card.stage());

                let attempt_info = CardAttemptInfo {
                    is_capture: (new_card.captures == (prev_captures + 1)),
                    total_attempts: new_card.attempts,
                    total_captures: new_card.captures,
                };

                grouped_card_attempts
                    .entry(prac_key)
                    .or_default()
                    .2
                    .insert(new_card.card, attempt_info);
            }
        }

        for (key, snapshot) in &self.practices {
            grouped_card_attempts.entry(*key).or_default().0 = snapshot.attempts;
        }

        for (key, snapshot) in &other.practices {
            grouped_card_attempts.entry(*key).or_default().1 = snapshot.attempts;
        }

        let mut ret: Vec<_> = grouped_card_attempts
            .into_iter()
            .filter_map(|(key, group)| {
                let (prev_practices, new_practices, attempted_cards) = group;
                let (difficulty, shot_type, stage) = key;

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
            .collect();

        ret.sort_unstable();

        ret
    }
}

#[derive(Debug)]
pub struct Update<'a, G: Game> {
    prev: FileSnapshot<G>,
    cur: &'a FileSnapshot<G>,
    events: Vec<UpdateEvent<G>>,
}

impl<'a, G: Game> Update<'a, G> {
    pub fn events(&self) -> impl Iterator<Item = &UpdateEvent<G>> {
        self.events.iter()
    }

    pub fn prev_snapshot(&self) -> &FileSnapshot<G> {
        &self.prev
    }

    pub fn cur_snapshot(&self) -> &'a FileSnapshot<G> {
        self.cur
    }
}

#[derive(Debug)]
pub struct UpdateStream<G: Game> {
    cur_snapshot: FileSnapshot<G>,
}

impl<G: Game> UpdateStream<G> {
    pub fn new(snapshot: FileSnapshot<G>) -> Self {
        Self {
            cur_snapshot: snapshot,
        }
    }

    pub fn cur_snapshot(&self) -> &FileSnapshot<G> {
        &self.cur_snapshot
    }

    pub fn update(&mut self, new_snapshot: FileSnapshot<G>) -> Option<Update<'_, G>> {
        if self.cur_snapshot.timestamp() < new_snapshot.timestamp() {
            let prev = std::mem::replace(&mut self.cur_snapshot, new_snapshot);
            let mut events = prev.get_updates(&self.cur_snapshot);
            events.sort_unstable();

            Some(Update {
                prev,
                events,
                cur: &self.cur_snapshot,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct SnapshotStream<G: Game> {
    game: G,
    last_modified: SystemTime,
    phantom: PhantomData<G>,
}

impl<G: Game> SnapshotStream<G> {
    pub async fn new(game: G) -> Result<Self, anyhow::Error> {
        fs::metadata(game.score_path())
            .await?
            .modified()
            .map(|last_modified| Self {
                game,
                last_modified,
                phantom: PhantomData,
            })
            .map_err(|e| e.into())
    }

    pub fn score_path(&self) -> &Path {
        self.game.score_path()
    }

    pub async fn read_snapshot_data(&mut self) -> Result<FileSnapshot<G>, anyhow::Error> {
        let timestamp = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        fs::read(self.game.score_path())
            .await
            .map_err(|e| e.into())
            .and_then(|data| FileSnapshot::new(&self.game, timestamp, Cursor::new(data)))
    }

    pub async fn refresh_snapshots(&mut self) -> Result<Option<FileSnapshot<G>>, anyhow::Error> {
        let cur_time = SystemTime::now();
        let mtime = fs::metadata(self.game.score_path()).await?.modified()?;

        if (mtime > self.last_modified)
            && cur_time
                .duration_since(mtime)
                .map(|d| d >= Duration::from_secs(2))
                .unwrap_or(false)
        {
            self.last_modified = mtime;
            self.read_snapshot_data().await.map(Some)
        } else {
            Ok(None)
        }
    }
}
