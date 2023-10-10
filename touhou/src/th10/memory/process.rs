use process_memory::{Memory, ProcessHandleExt, TryIntoProcessHandle};
use touhou_macros::define_memory;

use crate::memory::{Attached, ProcessAttached};
use crate::th10::Touhou10;

define_memory! {
    GameMemory {
        process_name = "th10",
        game = Touhou10,
        access = MemoryAccess,

        score: u32 @ [0x0047_4C44],
        power: u16 @ [0x0047_4C48], // displayed power = this * 0.05
        faith: u32 @ [0x0047_4C4C],
        lives: u32 @ [0x0047_4C70],
        continues_used: u32 @ [0x0047_4C90],
        extends: u32 @ [0x0047_4C9C],

        character: u32 @ [0x0047_4C68], // Reimu / Marisa
        character_subtype: u32 @ [0x0047_4C6C], // shot type A/B/C

        difficulty: u32 @ [0x0047_4C74],
        stage: u32 @ [0x0047_4C7C],

        game_state: u32 @ [0x0047_4C84],
        cur_frame: u32 @ [0x0047_4C88],
        game_state_frame: u32 @ [0x0047_4C8C], // ??
        practice_flag: u32 @ [0x0047_4CA0],
        replay_flag: u32 @ [0x0049_1C00],

        active_spell: u32 @ [0x0047_76F4, 0x03788],
        active_spell_status: u32 @ [0x0047_76F4, 0x0378C],
        active_spell_bonus: i32 @ [0x0047_76F4, 0x03790],
        boss_lifebars: u32 @ [0x0047_770C, 0x9E90],

        bgm_filename: [u8; 20] @ [0x0049_669C],

        menu_base_ptr: u32 @ [0x0047_784C],
        submenu_flag: u32 @ [0x0047_784C, 0x80],
        submenu_selection: u32 @ [0x0047_784C, 0x30]
    }
}
