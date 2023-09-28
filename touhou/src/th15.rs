//! Definitions specific to Touhou 15 (*Legacy of Lunatic Kingdom*).

use serde::{Deserialize, Serialize};
use touhou_macros::define_game;

mod spellcards;

pub use spellcards::SpellId;

define_game! {
    /// The fifteenth game in the series: *Touhou Kanjuden ~ Legacy of Lunatic Kingdom*.
    Touhou15 {
        type SpellID = SpellId;
        type ShotPower = Gen2(4);
        const GAME_ID = LoLK;

        /// The selectable shot types in Touhou 15.
        ShotType {
            Reimu,
            Marisa,
            Sanae,
            Reisen,
        }

        /// The selectable difficulty levels in Touhou 15.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Difficulty {
            Easy,
            Normal,
            Hard,
            Lunatic,
            Extra
        }

        /// The playable stages in Touhou 10.
        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Stage {
            One: "Stage 1",
            Two: "Stage 2",
            Three: "Stage 3",
            Four: "Stage 4",
            Five: "Stage 5",
            Six: "Stage 6",
            Extra: "Extra Stage"
        }
    }
}
