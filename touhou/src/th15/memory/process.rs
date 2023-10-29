use touhou_macros::define_memory;

use crate::memory::{Attached, ProcessAttached};
use crate::th15::Touhou15;

define_memory! {
    GameMemory {
        process_name = "th15",
        game = Touhou15,
        access = MemoryAccess,

        game_type: u32 @ [0x004E_9BEC],

        total_retries: u32 @ [0x004E_7594],
        chapter_retries: u32 @ [0x004E_75B8],

        score: u32 @ [0x004E_740C],
        continues_used: u32 @ [0x004E_7414],
        lives: u32 @  [0x004E_7450],
        life_fragments: u32 @  [0x004E_7454],
        bombs: u32 @  [0x004E_745C],
        bomb_fragments: u32 @  [0x004E_7460],
        power: u32 @ [0x4E7440], // 0-400

        character: u32 @ [0x004E_7404],
        difficulty: u32 @ [0x004E_7410],
        stage: u32 @ [0x004E_73F0],
        chapter: u32 @ [0x004E_73F8],
        chapter_frames: u32 @ [0x004E_7400],
        practice_flags: u32 @ [0x004E_7794], // 16 = practice selected, 32 = spell practice selected, 256 = Pointdevice enabled, 0 otherwise

        active_spell: u32 @ [0x004E_9A70, 0x74],
        active_spell_status: u32 @ [0x004E_9A70, 0x78],
        active_spell_bonus: u32 @ [0x004E_9A70, 0x7C],
        boss_lifebars: u32 @ [0x004E_9A8C, 0x178]
    }
}
