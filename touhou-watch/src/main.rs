#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sysinfo::{ProcessRefreshKind, System, SystemExt};
use tauri::Window;
use touhou::th07::SpellId;
use touhou::types::{SpellCardId, SpellCardInfo};
use touhou::{SpellCard, Touhou7};

mod location;
mod memory;
mod run;

use memory::{GameState, Touhou7Memory};
use run::{ActiveRun, UpdateResult};

fn watcher(window: Window) {
    let mut system = System::new();
    let mut cur_process: Option<Touhou7Memory> = None;
    let mut cur_run: Option<ActiveRun> = None;
    let mut first_read_time: Option<Instant> = None;

    window.emit("game-detached", ()).unwrap();

    loop {
        if let Some(th_proc) = cur_process.as_ref() {
            if !th_proc.is_running(&mut system) {
                window.emit("game-detached", ()).unwrap();
                cur_process = None;
                continue;
            }

            match (cur_run.take(), GameState::new(th_proc)) {
                (r, Err(e)) => {
                    window.emit("error", e.to_string()).unwrap();
                    cur_run = r;
                }
                (Some(active), Ok(state)) => {
                    let result = active.update(state);

                    window
                        .emit(
                            "run-update",
                            (result.is_finished(), result.run(), result.new_events()),
                        )
                        .unwrap();

                    if let UpdateResult::Continuing(active) = result {
                        cur_run = Some(active);
                    }
                }
                (
                    None,
                    Ok(GameState::InGame {
                        practice,
                        paused,
                        stage,
                        player,
                    }),
                ) => {
                    let now = Instant::now();
                    if let Some(t) = first_read_time {
                        if now.saturating_duration_since(t) >= Duration::from_millis(500) {
                            cur_run = ActiveRun::new(practice, paused, player, stage);
                            if let Some(active) = &cur_run {
                                window
                                    .emit("run-update", (false, active.run(), active.new_events()))
                                    .unwrap();

                                first_read_time = None;
                            }
                        }
                    } else {
                        first_read_time = Some(now);
                    }
                }
                (None, Ok(_)) => {}
            };

            sleep(Duration::from_millis(50));
        } else {
            system.refresh_processes_specifics(ProcessRefreshKind::new());
            cur_process = Touhou7Memory::new_autodetect(&system)
                .expect("could not initialize TH07 memory reader");

            if let Some(proc) = &cur_process {
                window.emit("game-attached", proc.pid()).unwrap();
            } else {
                sleep(Duration::from_millis(100))
            }
        }
    }
}

#[tauri::command]
fn load_spellcard_data() -> Vec<SpellCardInfo> {
    (1..=141)
        .map(|i| {
            let id: SpellId = i.try_into().unwrap();
            id.card_info()
        })
        .copied()
        .collect()
}

#[tauri::command]
fn init_events(window: Window) {
    std::thread::spawn(move || watcher(window));
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![init_events, load_spellcard_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
