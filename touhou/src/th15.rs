use serde::{Deserialize, Serialize};
use touhou_macros::define_game;

mod spellcards;

pub use spellcards::SpellId;

define_game! {
    Touhou15 {
        type SpellID = SpellId;
        const GAME_ID = LoLK;

        ShotType {
            Reimu,
            Marisa,
            Sanae,
            Reisen,
        }

        #[derive(Serialize, Deserialize)]
        #[serde(into = "u8", try_from = "u8")]
        Difficulty {
            Easy,
            Normal,
            Hard,
            Lunatic,
            Extra
        }

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
