use std::time::{Duration, SystemTime};
use std::{any, env};

use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqlitePool;
use sqlx::{Acquire, Sqlite};
use th07::Touhou7;
use time::OffsetDateTime;
use tokio::sync::{mpsc, oneshot};
use tokio::time::interval;
use types::{Difficulty, Game, Stage};

pub mod db;
pub mod decompress;
pub mod types;

pub mod th07;

use db::{CardAttemptInfo, CardSnapshot, SnapshotStream};

use crate::db::PracticeSnapshot;

pub async fn display_card_stats<G: Game>(
    snapshot: &CardSnapshot<G>,
    attempt_info: Option<&CardAttemptInfo>,
) -> anyhow::Result<()> {
    let title = format!("#{:03} {}", snapshot.card_id, snapshot.card_name());
    let update_status = (
        attempt_info.map(|a| a.is_capture()).unwrap_or(false),
        attempt_info.is_some(),
    );

    let capture_status = match update_status {
        (true, true) => " - CAPTURE",
        (false, true) => " - MISS",
        _ => "",
    };

    let key_str = match snapshot.stage() {
        Stage::Extra => format!("   Extra Stage  / {:<8}", snapshot.shot_type.to_string()),
        Stage::Phantasm => format!("Phantasm Stage  / {:<8}", snapshot.shot_type.to_string()),
        other => format!(
            "{} {:<7} / {:<8}",
            other,
            snapshot.difficulty().to_string(),
            snapshot.shot_type.to_string()
        ),
    };

    print!(
        "{:^85} [{}]: {:>4} / {:<4} ({:^5.1}%",
        title,
        key_str,
        snapshot.captures,
        snapshot.attempts,
        ((snapshot.captures as f64) / (snapshot.attempts as f64)) * 100.0
    );

    println!("){}", capture_status);

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

    let mut prev_snapshot = snap_stream.read_snapshot_data().await?;
    {
        let mut cards: Vec<_> = prev_snapshot.iter_cards().collect();
        cards.sort_unstable_by_key(|c| (c.card_id, c.shot_type));
        for card_snapshot in cards {
            display_card_stats(card_snapshot, None).await?;
        }
    }

    prev_snapshot.insert(&pool).await?;

    let mut interval = interval(Duration::from_millis(1000));
    while !ctrl_c_handle.is_finished() {
        let f = async {
            interval.tick().await;
            snap_stream.refresh_snapshots().await
        };

        if let Some((_, new_snapshot)) = tokio::select! {
            s = f => s?,
            _ = &mut ctrl_c_handle => {
                println!("Ctrl-C received, exiting...");
                break;
            }
        } {
            let mut updates = prev_snapshot.get_updates(&new_snapshot);
            updates.sort_unstable_by_key(|update| {
                (update.shot_type(), update.difficulty(), update.stage())
            });

            for update in updates {
                print!(
                    "\n[{}] {} {} {}",
                    update.timestamp(),
                    update.shot_type(),
                    update.stage(),
                    update.difficulty()
                );
                if let Some(practice_no) = update.practice_no() {
                    print!(" Practice #{}", practice_no);
                }
                println!(":");

                let mut attempted: Vec<_> = update.attempted_cards().collect();
                attempted.sort_unstable_by_key(|kv| kv.0);
                for (card_id, attempt_info) in attempted {
                    let new_card_snapshot =
                        new_snapshot.get_card(update.shot_type(), card_id).unwrap();
                    display_card_stats(new_card_snapshot, Some(attempt_info)).await?;
                }
            }

            new_snapshot.insert(&pool).await?;
            prev_snapshot = new_snapshot;
        }
    }

    pool.close().await;

    Ok(())
}
