use touhou_macros::define_memory;

use crate::memory::{Attached, ProcessAttached};
use crate::th07::Touhou7;

define_memory! {
    /// Provides access to the memory of a running Touhou 7 process (i.e. `th07.exe`).
    GameMemory {
        process_name = "th07",
        game = Touhou7,

        /// A helper struct for accessing the memory of a running Touhou 7 process.
        access = MemoryAccess,

        stage: u32 @ [0x0062f85c],
        menu_state: u32 @ [0x004b9e44, 0x0c],
        game_state: u32 @ [0x00575aa8],
        game_mode: u8 @ [0x0062f648],
        difficulty: u32 @ [0x00626280],
        ecl_time: u32 @ [0x009a9af8, 0x009545fc],
        spell_active: u32 @ [0x012fe0c8],
        spell_captured: u32 @ [0x012fe0c4],
        current_spell_id: u32 @ [0x012fe0d8],
        boss_flag: u32 @ [0x0049fc14],
        midboss_flag: u8 @ [0x009b655a],
        boss_id: u8 @ [0x009b1879],
        boss_healthbars: u32 @ [0x0049fc08],
        player_character: u8 @ [0x0062f647],
        player_lives: f32 @ [0x00626278, 0x5c],
        player_bombs: f32 @ [0x00626278, 0x68],
        player_power: f32 @ [0x00626278, 0x7c],
        player_misses: f32 @ [0x00626278, 0x50],
        player_bombs_used: f32 @ [0x00626278, 0x6c],
        player_continues: u8 @ [0x00626278, 0x20],
        border_state: u8 @ [0x004B_FEE5], // 0x004bdad8 + 0x240d
        score: u32 @ [0x00626278, 0x04],
        graze: u32 @ [0x00626278, 0x18],
        cherry_base: u32 @ [0x00626278, 0x88],
        cherry: u32 @ [0x0062f88c],
        cherry_max: u32 @ [0x0062f888],
        cherry_plus: u32 @ [0x0062f890],
    }
}
