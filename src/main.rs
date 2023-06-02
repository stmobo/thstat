use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::time::Duration;
use tokio::fs;

use th07::ShotType;
use time::OffsetDateTime;

pub mod decompress;
pub mod types;

pub mod th07;

use th07::score::{ScoreReader, Segment, SpellCardData};
use types::IterableEnum;

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub struct CardSnapshot {
    #[sqlx(rename = "ts")]
    timestamp: OffsetDateTime,
    card_id: u16,
    #[sqlx(try_from = "u8")]
    shot_type: ShotType,
    captures: u16,
    attempts: u16,
    max_bonus: u32,
}

impl CardSnapshot {
    pub fn from_score_data(
        timestamp: OffsetDateTime,
        data: &SpellCardData,
    ) -> impl Iterator<Item = CardSnapshot> + '_ {
        ShotType::iter_all().map(move |k| CardSnapshot {
            timestamp,
            card_id: data.card_id(),
            shot_type: k,
            captures: data.captures(&k),
            attempts: data.attempts(&k),
            max_bonus: data.max_bonuses(&k),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<_> = env::args().collect();
    let pool =
        SqlitePool::connect(&env::var("DATABASE_URL").unwrap_or(String::from("sqlite:touhou.db")))
            .await?;
    let mut last_modified = HashMap::new();
    let mut last_snapshots = HashMap::new();
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    if args.len() < 2 {
        eprintln!("USAGE: {} [path to score.dat]", args[0]);
        anyhow::bail!("incorrect program arguments");
    }

    let path = &args[1];
    loop {
        interval.tick().await;

        let mtime = fs::metadata(path).await?.modified()?;
        let should_snapshot = {
            if let Some(prev_mtime) = last_modified.insert(path.clone(), mtime) {
                mtime > prev_mtime
            } else {
                true
            }
        };

        if should_snapshot {
            let timestamp =
                OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            let mts: OffsetDateTime = mtime.into();
            println!("[{}] Score file {} updated at {}", timestamp, path, mts);

            let data = Cursor::new(fs::read(path).await?);
            let mut tx = pool.begin().await?;

            for segment in ScoreReader::new(data).unwrap() {
                if let Segment::SpellCard(data) = segment.unwrap() {
                    for snapshot in CardSnapshot::from_score_data(timestamp, &data) {
                        let shot_type: u8 = snapshot.shot_type.into();
                        sqlx::query!(
                                r#"
                                INSERT INTO spellcards (ts, card_id, shot_type, captures, attempts, max_bonus)
                                VALUES (?, ?, ?, ?, ?, ?)
                                "#,
                                snapshot.timestamp, snapshot.card_id, shot_type, snapshot.captures, snapshot.attempts, snapshot.max_bonus
                            ).execute(&mut tx).await?;

                        let data_updated = {
                            if let Some(prev_snapshot) = last_snapshots
                                .insert((snapshot.card_id, snapshot.shot_type), snapshot)
                            {
                                (prev_snapshot.captures != snapshot.captures)
                                    || (prev_snapshot.attempts != snapshot.attempts)
                            } else {
                                true
                            }
                        };

                        if (snapshot.attempts > 0) && data_updated {
                            println!(
                                "#{:03} {:^80} [{:<8}]: {:03} / {:03} ({:^5.1}%)",
                                snapshot.card_id,
                                data.card_name(),
                                snapshot.shot_type.to_string(),
                                snapshot.captures,
                                snapshot.attempts,
                                ((snapshot.captures as f64) / (snapshot.attempts as f64)) * 100.0
                            );
                        }
                    }
                }
            }

            tx.commit().await?;
        }
    }

    Ok(())
}
