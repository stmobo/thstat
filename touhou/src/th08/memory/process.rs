use process_memory::{Memory, ProcessHandleExt, TryIntoProcessHandle};
use touhou_macros::define_memory;

use super::state::GameState;
use crate::memory::{Attached, ProcessAttached};

define_memory! {
    GameMemory {
        process_name = "th08",
        access = MemoryAccess,

        program_state: u32 @ [0x017C_E8B4],
        menu_state: u32 @ [0x017C_E8B0],
        game_mode: u32 @ [0x0164_D0B4],

        character: u8 @ [0x0164_D0B1],
        difficulty: u8 @ [0x0160_F538],

        score_1: u32 @ [0x0160_F510, 0x08],
        score_2: u32 @ [0x0160_F510, 0x00],

        player_lives: f32 @ [0x0160_F510, 0x74],
        player_bombs: f32 @ [0x0160_F510, 0x80],
        player_power: f32 @ [0x0160_F510, 0x98],

        misses: f32 @ [0x0160_F510, 0x64],
        bombs_used: f32 @ [0x0160_F510, 0x84],
        continues_used: u8 @ [0x0160_F510, 0x29],

        graze_1: u32 @ [0x0160_F510, 0x04],
        graze_2: u32 @ [0x0160_F510, 0x0C],

        time_1: u32 @ [0x0164_CFB4],
        time_2: u32 @ [0x0160_F510, 0x3C],
        time_3: u32 @ [0x0160_F510, 0x44],

        value: u32 @ [0x0160_F510, 0x24],
        gauge: u16 @ [0x0160_F510, 0x22],
        night: u8 @ [0x0160_F510, 0x28],

        rank: u32 @ [0x0164_D334],

        frame: u32 @ [0x00F5_4CF8],
        stage: u8 @ [0x004E_4850],

        boss_active: u8 @ [0x018B_89B8],
        boss_healthbars: u32 @ [0x0120_F448],
        spell_prac_id: u16 @ [0x0164_D0B8],
        cur_spell_state: u32 @ [0x004E_A670],
        cur_spell_id: u32 @ [0x004E_A678],
    }
}

impl GameMemory {
    pub fn read_state(&mut self) -> std::io::Result<GameState> {
        self.access().and_then(GameState::new)
    }
}