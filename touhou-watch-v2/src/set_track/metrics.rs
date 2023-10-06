use std::sync::{LockResult, Mutex, MutexGuard, OnceLock};

use touhou::memory::GameLocation;
use touhou::th07::Location as Th07Location;
use touhou::th08::Location as Th08Location;
use touhou::th10::{Location as Th10Location, SpellId as Th10SpellId};
use touhou::types::GameId;
use touhou::{Location, SpellCard, Touhou10, Touhou7, Touhou8};

use super::SetTracker;

#[derive(Debug, Default)]
pub struct Metrics {
    th07: SetTracker<Touhou7>,
    th08: SetTracker<Touhou8>,
    th10: SetTracker<Touhou10>,
}

impl Metrics {
    fn new() -> Self {
        Self::default()
    }

    pub fn get() -> MetricsHandle {
        static INSTANCE: OnceLock<Mutex<Metrics>> = OnceLock::new();
        let r = INSTANCE.get_or_init(|| Mutex::new(Self::new()));
        MetricsHandle(r)
    }

    pub fn th07(&self) -> &SetTracker<Touhou7> {
        &self.th07
    }

    pub fn th07_mut(&mut self) -> &mut SetTracker<Touhou7> {
        &mut self.th07
    }

    pub fn th08(&self) -> &SetTracker<Touhou8> {
        &self.th08
    }

    pub fn th08_mut(&mut self) -> &mut SetTracker<Touhou8> {
        &mut self.th08
    }

    pub fn th10(&self) -> &SetTracker<Touhou10> {
        &self.th10
    }

    pub fn th10_mut(&mut self) -> &mut SetTracker<Touhou10> {
        &mut self.th10
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct MetricsHandle(&'static Mutex<Metrics>);

impl MetricsHandle {
    pub fn lock(&self) -> MutexGuard<'_, Metrics> {
        self.try_lock().expect("could not lock metrics mutex")
    }

    pub fn try_lock(&self) -> LockResult<MutexGuard<'_, Metrics>> {
        self.0.lock()
    }
}

fn start_tracking_th07(start_index: usize, end_index: usize) -> Result<(), &'static str> {
    let mut locs = Th07Location::iter_all().enumerate().filter_map(|(i, loc)| {
        if i == start_index || i == end_index {
            Some(Location::new(loc))
        } else {
            None
        }
    });

    let start = locs.next().ok_or("invalid start index")?;
    let end = locs.next().ok_or("invalid end index")?;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th07_mut().start_tracking(start, end);
    Ok(())
}

fn start_tracking_th08(start_index: usize, end_index: usize) -> Result<(), &'static str> {
    let mut locs = Th08Location::iter_all().enumerate().filter_map(|(i, loc)| {
        if i == start_index || i == end_index {
            Some(Location::new(loc))
        } else {
            None
        }
    });

    let start = locs.next().ok_or("invalid start index")?;
    let end = locs.next().ok_or("invalid end index")?;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th08_mut().start_tracking(start, end);
    Ok(())
}

fn start_tracking_th10(start_index: u64, end_index: u64) -> Result<(), &'static str> {
    let start_spell = start_index
        .try_into()
        .map_err(|_| "invalid start spell ID")?;
    let end_spell = end_index.try_into().map_err(|_| "invalid start spell ID")?;

    let start =
        Th10Location::from_spell(SpellCard::new(start_spell)).ok_or("invalid start spell")?;
    let end = Th10Location::from_spell(SpellCard::new(end_spell)).ok_or("invalid end spell")?;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th10_mut()
        .start_tracking(Location::new(start), Location::new(end));

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
struct LocationInfo {
    game: GameId,
    name: &'static str,
    stage: (u64, &'static str),
    index: u64,
}

impl LocationInfo {
    fn get_th07() -> impl Iterator<Item = Self> {
        Th07Location::iter_all()
            .enumerate()
            .map(|(index, location)| Self {
                game: GameId::PCB,
                name: location.name(),
                stage: (location.stage().into(), location.stage().name()),
                index: index as u64,
            })
    }

    fn get_th08() -> impl Iterator<Item = Self> {
        Th08Location::iter_all()
            .enumerate()
            .map(|(index, location)| Self {
                game: GameId::IN,
                name: location.name(),
                stage: (location.stage().into(), location.stage().name()),
                index: index as u64,
            })
    }

    fn get_th10() -> impl Iterator<Item = Self> {
        Th10SpellId::iter_all().map(|spell| Self {
            game: GameId::MoF,
            name: spell.name,
            stage: (spell.stage.into(), spell.stage.name()),
            index: spell.into(),
        })
    }
}

#[tauri::command]
fn get_locations(game_id: GameId) -> Result<Vec<LocationInfo>, &'static str> {
    match game_id {
        GameId::PCB => Ok(LocationInfo::get_th07().collect()),
        GameId::IN => Ok(LocationInfo::get_th08().collect()),
        GameId::MoF => Ok(LocationInfo::get_th10().collect()),
        _ => Err("game not supported for tracking"),
    }
}

#[tauri::command]
fn start_tracking(game_id: GameId, start_index: u64, end_index: u64) -> Result<(), &'static str> {
    match game_id {
        GameId::PCB => start_tracking_th07(start_index as usize, end_index as usize),
        GameId::IN => start_tracking_th08(start_index as usize, end_index as usize),
        GameId::MoF => start_tracking_th10(start_index, end_index),
        _ => Err("game not supported for tracking"),
    }
}

pub fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .invoke_handler(tauri::generate_handler![start_tracking])
        .invoke_handler(tauri::generate_handler![get_locations])
}
