use std::env;
use std::fs::File;
use std::path::PathBuf;

use touhou::score::{PracticeRecord, ScoreFile as ScoreFileTrait, SpellCardRecord};
use touhou::th08::ScoreFile;

fn main() -> Result<(), std::io::Error> {
    let file_path = env::args().nth(1).map(PathBuf::from).unwrap();

    let score_file = File::open(file_path).and_then(ScoreFile::new)?;
    for data in score_file.spell_cards() {
        let card = data.card();
        for shot in data.shot_types() {
            if data.attempts(shot) > 0 {
                println!(
                    "#{:03} {} [{}] - {} / {}",
                    card.id(),
                    card.name(),
                    shot,
                    data.captures(shot),
                    data.attempts(shot)
                );
            }
        }
    }

    for practice in score_file.practice_records() {
        println!(
            "{} {} {} - {} attempts (score {})",
            practice.difficulty(),
            practice.stage(),
            practice.shot_type(),
            practice.attempts(),
            practice.high_score()
        );
    }

    Ok(())
}
