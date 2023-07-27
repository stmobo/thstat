use std::env;
use std::time::Duration;

use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqlitePool;
use sqlx::{Acquire, Sqlite};
use th07::Touhou7;
use time::OffsetDateTime;
use tokio::sync::{mpsc, oneshot};
use types::Game;

pub mod db;
pub mod decompress;
pub mod types;

pub mod th07;

use db::{CardAttempt, CardSnapshot, SnapshotStream};
use th07::spellcard_names::resolve_card_name;

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

async fn snapshot_loop<G: Game>(
    mut conn: PoolConnection<Sqlite>,
    mut stream: SnapshotStream<G>,
    mut exit_rx: oneshot::Receiver<()>,
    event_tx: mpsc::UnboundedSender<(CardSnapshot<G>, CardSnapshot<G>)>,
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
            Ok(Some((mtime, snapshots))) => {
                let mtime: OffsetDateTime = mtime.into();
                println!(
                    "\n[{}] Score file {} updated",
                    mtime,
                    stream.score_path().display()
                );

                let mut tx = conn.begin().await?;

                for snapshot in snapshots {
                    let prev_snapshot: Option<CardSnapshot<G>> = CardSnapshot::get_last_snapshot(
                        &mut tx,
                        snapshot.card_id,
                        snapshot.shot_type,
                    )
                    .await?;

                    if let Some(prev_snapshot) = prev_snapshot {
                        if snapshot.attempts == (prev_snapshot.attempts + 1)
                            && event_tx.send((prev_snapshot, snapshot)).is_err()
                        {
                            return Ok(());
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

    let snap_stream: SnapshotStream<Touhou7> = tokio::select! {
        s = SnapshotStream::new() => s?,
        _ = &mut exit_rx => {
            return Ok(())
        }
    };

    for snapshot in snap_stream.read_card_data().await? {
        display_card_stats(&pool, snapshot, None).await?;
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
            display_card_stats(&pool, ev.1, Some(ev.0)).await?;
        } else {
            break;
        }
    }

    pool.close().await;
    join_handle.await.map_err(|e| e.into()).and_then(|e| e)
}
