//! Definitions specific to Touhou 7 (*Perfect Cherry Blossom*).

use serde::{Deserialize, Serialize};
#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, System, SystemExt};

#[cfg(feature = "memory")]
pub mod memory;

mod spellcards;

#[cfg(feature = "score-file")]
pub mod score;

#[cfg(feature = "tracking")]
pub use memory::tracking::Touhou7Event;
#[cfg(feature = "memory")]
pub use memory::{GameMemory, Location};
#[cfg(feature = "score-file")]
pub use score::{PracticeData, ScoreFile, SpellCardData};
pub use spellcards::SpellId;
use touhou_macros::define_game;

use crate::types::GameId;

define_game! {
    /// The seventh game in the series: *Touhou Youyoumu ~ Perfect Cherry Blossom*.
    Touhou7 {
        type SpellID = SpellId;
        type ShotPower = Gen1;
        const GAME_ID = PCB;

        /// The selectable shot types in Touhou 7.
        ShotType {
            ReimuA,
            ReimuB,
            MarisaA,
            MarisaB,
            SakuyaA,
            SakuyaB
        }

        /// The selectable difficulty levels in Touhou 7.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Difficulty {
            Easy,
            Normal,
            Hard,
            Lunatic,
            Extra,
            Phantasm,
        }

        /// The playable stages in Touhou 7.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Stage {
            One: "Stage 1",
            Two: "Stage 2",
            Three: "Stage 3",
            Four: "Stage 4",
            Five: "Stage 5",
            Six: "Stage 6",
            Extra: "Extra Stage",
            Phantasm: "Phantasm Stage",
        }
    }
}

#[cfg(feature = "score-file")]
impl Touhou7 {
    pub fn load_score_file<R: std::io::Read>(src: R) -> Result<score::ScoreFile, std::io::Error> {
        ScoreFile::new(src)
    }
}

#[cfg(feature = "find-process")]
impl Touhou7 {
    pub fn find_process(system: &System) -> Option<&Process> {
        system
            .processes()
            .iter()
            .map(|(_, process)| process)
            .find(|&process| {
                process
                    .exe()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("th07"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}
