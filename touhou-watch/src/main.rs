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

#[derive(Debug)]
enum WatcherState {
    Detached,
    WaitingForGame(Box<Touhou7Memory>),
    WaitingForFirstRead(Box<Touhou7Memory>),
    InGame(Box<Touhou7Memory>, Box<ActiveRun>),
}

fn update_watcher_state(state: WatcherState, window: &Window, system: &mut System) -> WatcherState {
    match state {
        WatcherState::Detached => loop {
            system.refresh_processes_specifics(ProcessRefreshKind::new());

            let proc = Touhou7Memory::new_autodetect(system)
                .expect("could not initialize TH07 memory reader");

            if let Some(proc) = proc {
                window.emit("game-attached", proc.pid()).unwrap();
                return WatcherState::WaitingForGame(Box::new(proc));
            }

            sleep(Duration::from_millis(100));
        },
        WatcherState::WaitingForGame(proc) => loop {
            if !proc.is_running(system) {
                window.emit("game-detached", ()).unwrap();
                return WatcherState::Detached;
            }

            match GameState::new(&proc) {
                Err(e) => window.emit("error", e.to_string()).unwrap(),
                Ok(GameState::InGame { .. }) => {
                    std::thread::sleep(Duration::from_millis(1000));
                    return WatcherState::WaitingForFirstRead(proc);
                }
                _ => {}
            }

            std::thread::sleep(Duration::from_millis(100))
        },
        WatcherState::WaitingForFirstRead(proc) => loop {
            if !proc.is_running(system) {
                window.emit("game-detached", ()).unwrap();
                return WatcherState::Detached;
            }

            match GameState::new(&proc) {
                Err(e) => window.emit("error", e.to_string()).unwrap(),
                Ok(GameState::InGame {
                    practice,
                    paused,
                    stage,
                    player,
                }) => {
                    if let Some(active) = ActiveRun::new(practice, paused, player, stage) {
                        window
                            .emit("run-update", (false, active.run(), active.new_events()))
                            .unwrap();
                        return WatcherState::InGame(proc, Box::new(active));
                    }
                }
                _ => return WatcherState::WaitingForGame(proc),
            }

            std::thread::sleep(Duration::from_millis(50));
        },
        WatcherState::InGame(proc, active) => {
            let mut active = { *active };

            loop {
                if !proc.is_running(system) {
                    window.emit("game-detached", ()).unwrap();
                    return WatcherState::Detached;
                }

                match GameState::new(&proc) {
                    Err(e) => window.emit("error", e.to_string()).unwrap(),
                    Ok(s) => {
                        let result = active.update(s);
                        window
                            .emit(
                                "run-update",
                                (result.is_finished(), result.run(), result.new_events()),
                            )
                            .unwrap();

                        if let UpdateResult::Continuing(r) = result {
                            active = r;
                        } else {
                            return WatcherState::WaitingForGame(proc);
                        }
                    }
                }

                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn watcher(window: Window) {
    let mut system = System::new();
    let mut cur_state = WatcherState::Detached;

    window.emit("game-detached", ()).unwrap();

    loop {
        cur_state = update_watcher_state(cur_state, &window, &mut system);
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
