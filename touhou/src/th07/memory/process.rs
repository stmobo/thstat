use std::io::Result as IOResult;

use process_memory::{Architecture, DataMember, Memory, ProcessHandleExt, TryIntoProcessHandle};
use sysinfo::{Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, System, SystemExt};

macro_rules! define_memory {
    {
        $struct_name:ident {
            $($field_name:ident: $field_type:ty => [$($addr:expr),+]),*$(,)?
        }
    } => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            pid: u32,
            $($field_name: DataMember<$field_type>),*
        }

        impl $struct_name {
            pub fn new(pid: u32) -> IOResult<Self> {
                let handle = pid
                    .try_into_process_handle()?
                    .set_arch(Architecture::Arch32Bit);

                Ok(Self {
                    pid,
                    $($field_name: DataMember::new_offset(handle, vec![$($addr),+])),*
                })
            }

            pub fn pid(&self) -> u32 {
                self.pid
            }

            $(
                pub fn $field_name(&self) -> IOResult<$field_type> {
                    unsafe { self.$field_name.read() }
                }
            )*
        }
    };
}

define_memory! {
    GameMemory {
        stage: u32 => [0x0062f85c],
        menu_state: u32 => [0x004b9e44, 0x0c],
        game_state: u32 => [0x00575aa8],
        game_mode: u8 => [0x0062f648],
        difficulty: u32 => [0x00626280],
        ecl_time: u32 => [0x009a9af8, 0x009545fc],
        spell_active: u32 => [0x012fe0c8],
        spell_captured: u32 => [0x012fe0c4],
        current_spell_id: u32 => [0x012fe0d8],
        boss_flag: u32 => [0x0049fc14],
        midboss_flag: u8 => [0x009b655a],
        boss_id: u8 => [0x009b1879],
        boss_healthbars: u32 => [0x0049fc08],
        player_character: u8 => [0x0062f647],
        player_lives: f32 => [0x00626278, 0x5c],
        player_bombs: f32 => [0x00626278, 0x68],
        player_power: f32 => [0x00626278, 0x7c],
        player_misses: f32 => [0x00626278, 0x50],
        player_bombs_used: f32 => [0x00626278, 0x6c],
        player_continues: u8 => [0x00626278, 0x20],
        border_state: u8 => [0x004bdad8 + 0x240d],
        score: u32 => [0x00626278, 0x04],
        graze: u32 => [0x00626278, 0x18],
        cherry_base: u32 => [0x00626278, 0x88],
        cherry: u32 => [0x0062f88c],
        cherry_max: u32 => [0x0062f888],
        cherry_plus: u32 => [0x0062f890],
    }
}

impl GameMemory {
    fn find_process(system: &System) -> Option<&Process> {
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
            .filter(|proc| proc.run_time() > 15)
    }

    pub fn new_autodetect(system: &System) -> IOResult<Option<Self>> {
        if let Some(proc) = Self::find_process(system) {
            Self::new(proc.pid().as_u32()).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn is_running(&self, system: &mut System) -> bool {
        system.refresh_process_specifics(Pid::from_u32(self.pid), ProcessRefreshKind::new())
    }
}
