//! Definitions specific to Touhou 8 (*Imperishable Night*).

use serde::{Deserialize, Serialize};
#[cfg(feature = "find-process")]
use sysinfo::{Process, ProcessExt, System, SystemExt};
use touhou_macros::define_game;

#[cfg(feature = "score-file")]
pub mod score;

mod spellcards;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "memory")]
pub use memory::{GameMemory, Location};
#[cfg(feature = "score-file")]
pub use score::ScoreFile;
pub use spellcards::SpellId;

define_game! {
    /// The eighth game in the series: *Touhou Eiyashou ~ Imperishable Night*.
    Touhou8 {
        type SpellID = SpellId;
        type ShotPower = Gen1;
        const GAME_ID = IN;

        /// The selectable shot types in Touhou 8.
        ShotType {
            BarrierTeam: "Reimu & Yukari",
            MagicTeam: "Marisa & Alice",
            ScarletTeam: "Sakuya & Remilia",
            GhostTeam: "Youmu & Yuyuko",
            Reimu,
            Yukari,
            Marisa,
            Alice,
            Sakuya,
            Remilia,
            Youmu,
            Yuyuko,
        }

        /// The selectable difficulty levels in Touhou 8.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Difficulty {
            Easy,
            Normal,
            Hard,
            Lunatic,
            Extra,
            LastWord,
        }

        /// The playable stages in Touhou 8.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Stage {
            One: "Stage 1",
            Two: "Stage 2",
            Three: "Stage 3",
            FourA: "Stage 4 Uncanny",
            FourB: "Stage 4 Powerful",
            Five: "Stage 5",
            FinalA,
            FinalB,
            Extra: "Extra Stage",
            LastWord
        }
    }
}

#[cfg(feature = "find-process")]
impl Touhou8 {
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
                    .map(|s| s.starts_with("th08"))
                    .unwrap_or(false)
            })
    }

    pub fn find_score_file(proc: &Process) -> std::path::PathBuf {
        proc.exe().with_file_name("score.dat")
    }
}
