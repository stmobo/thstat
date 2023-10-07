use serde::Serialize;
use touhou::memory::GameLocation;
use touhou::types::{GameId, GameValue};
use touhou::Location;

use super::set_track::{Attempt, Metrics, SetKey};
use super::TrackedGame;
use crate::watcher::GameReader;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct NamedValue<T> {
    value: T,
    name: &'static str,
}

impl<T> NamedValue<T> {
    pub fn new<V: GameValue<RawValue = T>>(value: V) -> Self {
        Self {
            value: value.raw_id(),
            name: value.name(),
        }
    }
}

impl<T: GameValue> From<T> for NamedValue<T::RawValue> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SerializedLocation {
    name: &'static str,
    value: u64,
    stage: NamedValue<u16>,
    spell: Option<NamedValue<u32>>,
}

impl SerializedLocation {
    pub fn new<G: TrackedGame>(value: Location<G>) -> Self {
        Self {
            name: value.name(),
            value: value.index(),
            stage: value.stage().unwrap().into(),
            spell: value.spell().map(|wrapper| wrapper.unwrap().into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SerializedGameId {
    value: u8,
    abbreviation: &'static str,
    name: &'static str,
}

impl SerializedGameId {
    pub fn new(id: GameId) -> Self {
        Self {
            value: id.into(),
            name: id.subtitle(),
            abbreviation: id.abbreviation(),
        }
    }
}

impl From<GameId> for SerializedGameId {
    fn from(value: GameId) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SetInfo {
    game: SerializedGameId,
    shot_type: NamedValue<u16>,
    difficulty: NamedValue<u16>,
    location: SerializedLocation,
    attempts: Vec<Attempt>,
}

impl SetInfo {
    pub fn new<G: TrackedGame>(key: &SetKey<G>, attempts: &[Attempt]) -> Self {
        Self {
            game: SerializedGameId::new(G::GAME_ID),
            shot_type: key.shot().unwrap().into(),
            difficulty: key.difficulty().unwrap().into(),
            location: SerializedLocation::new(key.location()),
            attempts: attempts.into(),
        }
    }

    pub fn get_sets<G: TrackedGame>(metrics: &Metrics) -> impl Iterator<Item = SetInfo> + '_ {
        G::get_tracker(metrics)
            .iter_attempts()
            .map(|(k, v)| Self::new(k, v))
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AttachEvent {
    game: SerializedGameId,
    pid: u32,
}

impl AttachEvent {
    pub fn new(game_id: GameId, pid: u32) -> Self {
        Self {
            game: game_id.into(),
            pid,
        }
    }

    pub fn from_reader<G: TrackedGame>(reader: &G::Reader) -> Self {
        Self {
            game: G::GAME_ID.into(),
            pid: reader.pid(),
        }
    }
}
