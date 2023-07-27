use std::time::{Duration, SystemTime};
use std::{any, env};

use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqlitePool;
use sqlx::{Acquire, Sqlite};
use th07::Touhou7;
use time::OffsetDateTime;
use tokio::sync::{mpsc, oneshot};
use types::{Difficulty, Game, Stage};

pub mod db;
pub mod decompress;
pub mod types;

pub mod th07;

use db::{CardAttempt, CardSnapshot, SnapshotStream};
use th07::spellcard_names::resolve_card_name;

use crate::db::PracticeSnapshot;

pub struct ExponentialSmoothing<T> {
    src: T,
    cur: Option<f64>,
    alpha: f64,
}

impl<T: Iterator<Item = f64>> ExponentialSmoothing<T> {
    pub fn new(src: T, alpha: f64) -> Self {
        Self {
            src,
            alpha,
            cur: None,
        }
    }
}

impl<T: Iterator<Item = f64>> Iterator for ExponentialSmoothing<T> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(val) = self.src.next() {
            if let Some(cur) = &mut self.cur {
                *cur += self.alpha * (val - *cur);
            } else {
                self.cur = Some(val);
            }

            self.cur
        } else {
            None
        }
    }
}

pub async fn display_card_stats<G: Game>(
    pool: &SqlitePool,
    snapshot: CardSnapshot<G>,
    prev_snapshot: Option<CardSnapshot<G>>,
) -> anyhow::Result<()> {
    if let Some(card_name) = resolve_card_name(snapshot.card_id - 1) {
        let prev_attempts: Vec<CardAttempt<G>> =
            CardAttempt::get_card_attempts(pool, snapshot.card_id, snapshot.shot_type, 30).await?;

        let mut k = prev_attempts.len();
        let mut recent_cap_rate: u32 = prev_attempts
            .into_iter()
            .map(|attempt| attempt.captured as u32)
            .sum();

        let title = format!("#{:03} {}", snapshot.card_id, card_name);
        let update_status = prev_snapshot.map(|prev| {
            (
                (snapshot.captures == (prev.captures + 1)),
                (snapshot.attempts == (prev.attempts + 1)),
            )
        });

        let capture_status = match update_status {
            Some((true, true)) => {
                k += 1;
                recent_cap_rate += 1;
                " - CAPTURE"
            }
            Some((false, true)) => {
                k += 1;
                " - MISS"
            }
            _ => "",
        };

        print!(
            "{:^85} [{:<8}]: {:03} / {:03} ({:^5.1}%",
            title,
            snapshot.shot_type.to_string(),
            snapshot.captures,
            snapshot.attempts,
            ((snapshot.captures as f64) / (snapshot.attempts as f64)) * 100.0
        );

        if k > 0 {
            let k = k as f64;
            print!(", recent {:5.1}%", ((recent_cap_rate as f64) / k) * 100.0);
        }

        println!("){}", capture_status);
    }

    Ok(())
}

pub struct CardUpdate<G: Game> {
    pub prev: CardSnapshot<G>,
    pub new: CardSnapshot<G>,
}

impl<G: Game> CardUpdate<G> {
    pub fn is_attempt(&self) -> bool {
        self.new.attempts == (self.prev.attempts + 1)
    }

    pub fn is_capture(&self) -> bool {
        self.new.captures == (self.prev.captures + 1)
    }
}

pub struct SnapshotUpdate<G: Game> {
    pub mtime: OffsetDateTime,
    pub practice_type: Option<(Difficulty, G::ShotType, Stage, u32)>,
    pub card_updates: Vec<CardUpdate<G>>,
}

async fn snapshot_loop<G: Game>(
    mut conn: PoolConnection<Sqlite>,
    mut stream: SnapshotStream<G>,
    mut exit_rx: oneshot::Receiver<()>,
    event_tx: mpsc::UnboundedSender<SnapshotUpdate<G>>,
) -> Result<(), anyhow::Error> {
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    while exit_rx.try_recv() == Err(oneshot::error::TryRecvError::Empty) {
        tokio::select! {
            _ = interval.tick() => {},
            _ = &mut exit_rx => {
                return Ok(());
            }
        };

        match stream.refresh_snapshots().await {
            Ok(Some((mtime, mut score_snapshot))) => {
                let mut card_updates = Vec::new();
                let mut tx = conn.begin().await?;

                let mut practice_type = None;
                for snapshot in score_snapshot.practices.drain(..) {
                    let prev_snapshot: Option<PracticeSnapshot<G>> =
                        PracticeSnapshot::get_last_snapshot(
                            &mut tx,
                            snapshot.difficulty,
                            snapshot.shot_type,
                            snapshot.stage,
                        )
                        .await?;

                    if let Some(prev_snapshot) = prev_snapshot {
                        if snapshot.attempts == (prev_snapshot.attempts + 1) {
                            practice_type = Some((
                                snapshot.difficulty,
                                snapshot.shot_type,
                                snapshot.stage,
                                snapshot.attempts,
                            ));
                        }
                    }

                    let shot_type: u8 = snapshot.shot_type.into();
                    let difficulty: u8 = snapshot.difficulty.into();
                    let stage: u8 = snapshot.stage.into();

                    sqlx::query!(
                        r#"
                        INSERT INTO practices (ts, difficulty, shot_type, stage, attempts, high_score)
                        VALUES (?, ?, ?, ?, ?, ?)
                        "#,
                        snapshot.timestamp,
                        difficulty,
                        shot_type,
                        stage,
                        snapshot.attempts,
                        snapshot.high_score
                    )
                    .execute(&mut tx)
                    .await?;
                }

                for snapshot in score_snapshot.cards.drain(..) {
                    let prev_snapshot: Option<CardSnapshot<G>> = CardSnapshot::get_last_snapshot(
                        &mut tx,
                        snapshot.card_id,
                        snapshot.shot_type,
                    )
                    .await?;

                    if let Some(prev_snapshot) = prev_snapshot {
                        if (prev_snapshot.attempts != snapshot.attempts)
                            || (prev_snapshot.captures != snapshot.captures)
                        {
                            card_updates.push(CardUpdate {
                                prev: prev_snapshot,
                                new: snapshot,
                            });
                        }
                    }

                    let shot_type: u8 = snapshot.shot_type.into();
                    sqlx::query!(
                        r#"
                        INSERT INTO spellcards (ts, card_id, shot_type, captures, attempts, max_bonus)
                        VALUES (?, ?, ?, ?, ?, ?)
                        "#,
                        snapshot.timestamp,
                        snapshot.card_id,
                        shot_type,
                        snapshot.captures,
                        snapshot.attempts,
                        snapshot.max_bonus
                    )
                    .execute(&mut tx)
                    .await?;
                }

                tx.commit().await?;

                if !card_updates.is_empty()
                    && event_tx
                        .send(SnapshotUpdate {
                            mtime: mtime.into(),
                            practice_type,
                            card_updates,
                        })
                        .is_err()
                {
                    return Ok(());
                }
            }
            Ok(None) => continue,
            Err(e) => {
                eprintln!("Encountered error when reading score data: {:?}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let pool =
        SqlitePool::connect(&env::var("DATABASE_URL").unwrap_or(String::from("sqlite:touhou.db")))
            .await?;

    let (exit_tx, mut exit_rx) = oneshot::channel();
    let mut ctrl_c_handle = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        exit_tx.send(()).unwrap();
    });

    let mut snap_stream: SnapshotStream<Touhou7> = tokio::select! {
        s = SnapshotStream::new() => s?,
        _ = &mut exit_rx => {
            return Ok(())
        }
    };

    {
        let snap_data = snap_stream.read_snapshot_data().await?;
        for card_snapshot in snap_data.cards {
            display_card_stats(&pool, card_snapshot, None).await?;
        }
    }

    let (ev_tx, mut ev_rx) = mpsc::unbounded_channel();
    let join_handle = tokio::spawn(snapshot_loop(
        pool.acquire().await?,
        snap_stream,
        exit_rx,
        ev_tx,
    ));

    while !ctrl_c_handle.is_finished() {
        let ev = tokio::select! {
            ev = ev_rx.recv() => ev,
            _ = &mut ctrl_c_handle => {
                println!("Ctrl-C received, exiting...");
                break;
            }
        };

        if let Some(ev) = ev {
            print!("\n[{}] ", ev.mtime);

            if let Some((difficulty, shot, stage, attempts)) = ev.practice_type {
                print!("{} {} {} Practice #{}", shot, difficulty, stage, attempts);
            } else {
                print!("Score file updated")
            }

            if !ev.card_updates.is_empty() {
                println!(":");
                for card_update in ev.card_updates {
                    display_card_stats(&pool, card_update.new, Some(card_update.prev)).await?;
                }
            } else {
                println!();
            }
        } else {
            break;
        }
    }

    pool.close().await;
    join_handle.await.map_err(|e| e.into()).and_then(|e| e)
}
