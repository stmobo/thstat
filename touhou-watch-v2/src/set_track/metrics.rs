use core::panic;
use std::collections::{HashMap, HashSet};
use std::sync::{LockResult, Mutex, MutexGuard, OnceLock};

use touhou::memory::GameLocation;
use touhou::th07::Location as Th07Location;
use touhou::th08::Location as Th08Location;
use touhou::th10::{Location as Th10Location, SpellId as Th10SpellId};
use touhou::types::GameId;
use touhou::{
    AllIterable, GameValue, HasLocations, Location, SpellCard, Touhou10, Touhou7, Touhou8,
};

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

fn ensure_index_ordering(start: usize, end: usize) -> (usize, usize) {
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

pub fn start_tracking_th07(start_index: usize, end_index: usize) -> Result<(), &'static str> {
    let locs = LocationInfo::get_th07();
    let (start_index, end_index) = ensure_index_ordering(start_index, end_index);

    let start = locs
        .get(start_index)
        .ok_or("invalid start index")?
        .actual
        .unwrap_pcb()
        .0;

    let end = locs
        .get(end_index)
        .ok_or("invalid end index")?
        .actual
        .unwrap_pcb()
        .1;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th07_mut().start_tracking(start, end);
    Ok(())
}

pub fn start_tracking_th08(start_index: usize, end_index: usize) -> Result<(), &'static str> {
    let locs = LocationInfo::get_th08();
    let (start_index, end_index) = ensure_index_ordering(start_index, end_index);

    let start = locs
        .get(start_index)
        .ok_or("invalid start index")?
        .actual
        .unwrap_in()
        .0;

    let end = locs
        .get(end_index)
        .ok_or("invalid end index")?
        .actual
        .unwrap_in()
        .1;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th08_mut().start_tracking(start, end);
    Ok(())
}

pub fn start_tracking_th10(start_index: usize, end_index: usize) -> Result<(), &'static str> {
    let locs = LocationInfo::get_th10();
    let (start_index, end_index) = ensure_index_ordering(start_index, end_index);

    let start = locs
        .get(start_index)
        .ok_or("invalid start index")?
        .actual
        .unwrap_mof()
        .0;

    let end = locs
        .get(end_index)
        .ok_or("invalid end index")?
        .actual
        .unwrap_mof()
        .1;

    let metrics = Metrics::get();
    let mut lock = metrics.lock();
    lock.th10_mut().start_tracking(start, end);

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
enum StoredLocation {
    PCB(Location<Touhou7>, Location<Touhou7>),
    IN(Location<Touhou8>, Location<Touhou8>),
    MoF(Location<Touhou10>, Location<Touhou10>),
}

impl StoredLocation {
    fn unwrap_pcb(self) -> (Location<Touhou7>, Location<Touhou7>) {
        if let Self::PCB(a, b) = self {
            (a, b)
        } else {
            panic!("contained value was not a PCB location")
        }
    }

    fn unwrap_in(self) -> (Location<Touhou8>, Location<Touhou8>) {
        if let Self::IN(a, b) = self {
            (a, b)
        } else {
            panic!("contained value was not an IN location")
        }
    }

    fn unwrap_mof(self) -> (Location<Touhou10>, Location<Touhou10>) {
        if let Self::MoF(a, b) = self {
            (a, b)
        } else {
            panic!("contained value was not a MoF location")
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocationInfo {
    game: GameId,
    name: &'static str,
    stage: (u64, &'static str),
    #[serde(skip)]
    actual: StoredLocation,
}

impl LocationInfo {
    fn map_location_info<G, F, I>(mapper: F, iter: I) -> Vec<LocationInfo>
    where
        G: HasLocations,
        F: Fn(Location<G>, Location<G>) -> StoredLocation,
        I: IntoIterator<Item = Location<G>>,
    {
        let mut ranges: HashMap<_, (Location<G>, Location<G>)> = HashMap::new();
        for location in iter {
            let key = (location.stage(), location.name());
            let stored = ranges.entry(key).or_insert((location, location));
            stored.0 = <Location<G> as Ord>::min(stored.0, location);
            stored.1 = <Location<G> as Ord>::max(stored.1, location);
        }

        let mut loc = ranges
            .into_iter()
            .map(move |((stage, name), (min, max))| Self {
                game: G::GAME_ID,
                name,
                stage: (min.stage().unwrap().raw_id() as u64, stage.name()),
                actual: mapper(min, max),
            })
            .collect::<Vec<_>>();

        loc.sort_unstable_by(|a, b| a.actual.cmp(&b.actual));

        loc
    }

    pub fn get_th07() -> &'static [Self] {
        static LOCATIONS: OnceLock<Vec<LocationInfo>> = OnceLock::new();

        LOCATIONS.get_or_init(|| {
            Self::map_location_info(
                StoredLocation::PCB,
                Th07Location::iter_all().map(Location::new),
            )
        })
    }

    pub fn get_th08() -> &'static [Self] {
        static LOCATIONS: OnceLock<Vec<LocationInfo>> = OnceLock::new();

        LOCATIONS.get_or_init(|| {
            Self::map_location_info(
                StoredLocation::IN,
                Th08Location::iter_all().map(Location::new),
            )
        })
    }

    pub fn get_th10() -> &'static [Self] {
        static LOCATIONS: OnceLock<Vec<LocationInfo>> = OnceLock::new();
        LOCATIONS.get_or_init(|| {
            Self::map_location_info(
                StoredLocation::MoF,
                Th10SpellId::iter_all()
                    .map(SpellCard::new)
                    .filter_map(Location::from_spell),
            )
        })
    }
}
