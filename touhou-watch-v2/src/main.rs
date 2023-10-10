#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

use set_track::Metrics;
use tauri::Window;

mod event_serialize;
// mod persist;
mod set_track;
mod th07;
mod th08;
mod th10;
mod time;
mod watcher;

use event_serialize::SetInfo;
use set_track::LocationInfo;
use touhou::types::{GameId, SpellCardInfo};
use touhou::{AllIterable, Touhou10, Touhou7, Touhou8};
use watcher::TrackedGame;

#[derive(Debug, Clone, serde::Serialize)]
struct SpellCardData {
    th07: Vec<&'static SpellCardInfo<Touhou7>>,
    th08: Vec<&'static SpellCardInfo<Touhou8>>,
    th10: Vec<&'static SpellCardInfo<Touhou10>>,
}

#[tauri::command]
fn load_spellcard_data() -> SpellCardData {
    use touhou::th07::SpellId as PCBSpellId;
    use touhou::th08::SpellId as INSpellId;
    use touhou::th10::SpellId as MoFSpellId;

    SpellCardData {
        th07: PCBSpellId::iter_all().map(|id| id.card_info()).collect(),
        th08: INSpellId::iter_all().map(|id| id.card_info()).collect(),
        th10: MoFSpellId::iter_all().map(|id| id.card_info()).collect(),
    }
}

#[tauri::command]
fn start_watcher(window: Window) {
    let w2 = window.clone();
    let w3 = window.clone();
    std::thread::spawn(move || watcher::track_game::<Touhou7>(window));
    std::thread::spawn(move || watcher::track_game::<Touhou8>(w2));
    std::thread::spawn(move || watcher::track_game::<Touhou10>(w3));
}

#[tauri::command]
fn get_practice_data(game_id: Option<GameId>) -> Result<Vec<SetInfo>, &'static str> {
    let metrics = Metrics::get();
    let lock = metrics.lock();

    match game_id {
        None => Ok(SetInfo::get_sets::<Touhou7>(&lock)
            .chain(SetInfo::get_sets::<Touhou8>(&lock))
            .chain(SetInfo::get_sets::<Touhou10>(&lock))
            .collect()),
        Some(GameId::PCB) => Ok(SetInfo::get_sets::<Touhou7>(&lock).collect()),
        Some(GameId::IN) => Ok(SetInfo::get_sets::<Touhou8>(&lock).collect()),
        Some(GameId::MoF) => Ok(SetInfo::get_sets::<Touhou10>(&lock).collect()),
        Some(_) => Err("game not supported for tracking"),
    }
}

#[tauri::command]
fn get_locations(game_id: GameId) -> Result<&'static [LocationInfo], &'static str> {
    match game_id {
        GameId::PCB => Ok(LocationInfo::get_th07()),
        GameId::IN => Ok(LocationInfo::get_th08()),
        GameId::MoF => Ok(LocationInfo::get_th10()),
        _ => Err("game not supported for tracking"),
    }
}

#[tauri::command]
fn start_tracking(
    game_id: GameId,
    start_index: usize,
    end_index: usize,
) -> Result<(), &'static str> {
    match game_id {
        GameId::PCB => set_track::start_tracking_th07(start_index, end_index),
        GameId::IN => set_track::start_tracking_th08(start_index, end_index),
        GameId::MoF => set_track::start_tracking_th10(start_index, end_index),
        _ => Err("game not supported for tracking"),
    }
}

#[tauri::command]
fn end_tracking() {
    let metrics = Metrics::get();
    let mut lock = metrics.lock();

    lock.th07_mut().end_tracking();
    lock.th08_mut().end_tracking();
    lock.th10_mut().end_tracking();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_watcher,
            load_spellcard_data,
            get_practice_data,
            start_tracking,
            end_tracking,
            get_locations
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
