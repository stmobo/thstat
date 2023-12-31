use std::collections::HashMap;
use std::time::Instant;

use time::Duration;
use touhou::memory::{Location, PauseState, PlayerData, RunData, SpellState, StageData};
use touhou::{Difficulty, ShotType};

use super::{Attempt, Metrics, SetKey};
use crate::time::{GameTime, GameTimeCounter};
use crate::watcher::TrackedGame;

#[derive(Debug, Clone)]
pub struct LocationResolveFilter<G: TrackedGame> {
    last_location: (Instant, Location<G>),
    actual_location: Option<(Instant, Location<G>)>,
}

impl<G: TrackedGame> LocationResolveFilter<G> {
    const MIN_LOCATION_TIME: u64 = 1000; // ms

    pub fn new(location: Location<G>) -> Self {
        Self {
            last_location: (Instant::now(), location),
            actual_location: None,
        }
    }

    pub fn actual_location(&self) -> Option<Location<G>> {
        self.actual_location.map(|pair| pair.1)
    }

    pub fn update_location(&mut self, location: Location<G>) -> bool {
        use std::time::Duration as StdDuration;

        let now = Instant::now();

        if self.last_location.1 != location {
            self.last_location = (now, location);
            false
        } else {
            if self.actual_location.is_some_and(|pair| pair.1 == location) {
                return false;
            }

            let rel_time = now
                .checked_duration_since(self.last_location.0)
                .unwrap_or(StdDuration::ZERO);

            if rel_time >= StdDuration::from_millis(Self::MIN_LOCATION_TIME) {
                self.actual_location = Some(self.last_location);
                true
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SpellTracker<G: TrackedGame>(Option<SpellState<G>>);

impl<G: TrackedGame> SpellTracker<G> {
    pub fn new<T: StageData<G>>(state: &T) -> Self {
        Self(state.active_spell())
    }

    pub fn update<T: StageData<G>>(&mut self, state: &T) -> Option<SpellState<G>> {
        let cur_spell_state = state.active_spell();
        let prev_spell_state = std::mem::replace(&mut self.0, cur_spell_state);

        match (prev_spell_state, cur_spell_state) {
            (Some(prev), Some(cur)) => {
                if prev.spell() != cur.spell() {
                    Some(prev)
                } else {
                    None
                }
            }
            (Some(prev), None) => Some(prev),
            (None, None) | (None, Some(_)) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveAttempt<G: TrackedGame> {
    start_time: GameTime,
    location: Location<G>,
    success: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveGame<G: TrackedGame> {
    time_counter: GameTimeCounter,
    shot: ShotType<G>,
    difficulty: Difficulty<G>,
    spell_tracker: Option<SpellState<G>>,
    location_filter: Option<LocationResolveFilter<G>>,
    cur_attempt: Option<ActiveAttempt<G>>,
    attempts: Vec<(SetKey<G>, Attempt)>,
    cleared: bool,
}

impl<G: TrackedGame> ActiveGame<G> {
    pub fn new<T: RunData<G>>(run: &T) -> Self {
        ActiveGame {
            time_counter: GameTimeCounter::default(),
            shot: run.player().shot(),
            difficulty: run.difficulty(),
            spell_tracker: run.stage().active_spell(),
            location_filter: None,
            cur_attempt: None,
            attempts: Vec::new(),
            cleared: false,
        }
    }

    pub fn cur_location(&self) -> Option<Location<G>> {
        self.location_filter
            .as_ref()
            .and_then(LocationResolveFilter::actual_location)
    }

    fn push_attempt(&mut self) {
        if let Some(ActiveAttempt {
            start_time,
            location,
            success,
        }) = self.cur_attempt.take()
        {
            let key = SetKey::new(self.shot, self.difficulty, location);
            let end_time = self.time_counter.now();

            let metrics = Metrics::get();
            let mut lock = metrics.lock();
            G::get_tracker_mut(&mut lock)
                .push_attempt(key, Attempt::new(start_time, end_time, success));
        }
    }

    pub fn update_pause<T: PauseState>(&mut self, state: &T) {
        self.time_counter.update_from_state(state)
    }

    pub fn update_spell<T: StageData<G>>(&mut self, state: &T) -> bool {
        let cur_spell = state.active_spell();
        let prev_spell = std::mem::replace(&mut self.spell_tracker, cur_spell);
        let location_spell = self.cur_location().as_ref().and_then(Location::spell);

        match (prev_spell, cur_spell) {
            (Some(prev), Some(cur)) => {
                if prev.spell() != cur.spell() {
                    if location_spell.is_some_and(|spell| spell == prev.spell()) {
                        self.push_attempt();
                        return true;
                    }
                } else if prev.captured() && !cur.captured() {
                    self.mark_failed();
                    return true;
                }

                false
            }
            (Some(prev), None) => {
                if location_spell.is_some_and(|spell| spell == prev.spell()) {
                    self.push_attempt();
                    true
                } else {
                    false
                }
            }
            (None, None) | (None, Some(_)) => false,
        }
    }

    pub fn update_location(&mut self, location: Location<G>) -> bool {
        if let Some(filter) = &mut self.location_filter {
            if filter.update_location(location) {
                self.push_attempt();

                self.cur_attempt = Some(ActiveAttempt {
                    start_time: self.time_counter.now(),
                    location,
                    success: true,
                });

                true
            } else {
                false
            }
        } else {
            self.location_filter = Some(LocationResolveFilter::new(location));
            false
        }
    }

    pub fn exit_location(&mut self) {
        self.push_attempt();
        self.location_filter = None;
    }

    fn set_success(&mut self, value: bool) {
        if let Some(attempt) = &mut self.cur_attempt {
            attempt.success = value;
        }
    }

    pub fn mark_failed(&mut self) {
        self.set_success(false);
    }

    pub fn mark_cleared(&mut self, cleared: bool) {
        self.cleared = cleared;
    }
}

impl<G: TrackedGame> Drop for ActiveGame<G> {
    fn drop(&mut self) {
        if !self.cleared {
            self.mark_failed();
        }

        self.push_attempt();
    }
}

#[derive(Debug, Clone, Default)]
pub struct SetTracker<G: TrackedGame> {
    attempts: HashMap<SetKey<G>, Vec<Attempt>>,
    track_range: Option<(Location<G>, Location<G>)>,
}

impl<G: TrackedGame> SetTracker<G> {
    pub fn start_tracking(&mut self, start: Location<G>, end: Location<G>) {
        let range = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        eprintln!(
            "Starting set filtering for {} from {} to {}",
            G::GAME_ID.abbreviation(),
            range.0,
            range.1
        );

        self.track_range = Some(range);
    }

    pub fn end_tracking(&mut self) {
        eprintln!("Ending set filtering for {}", G::GAME_ID.abbreviation());

        self.track_range = None;
    }

    pub fn iter_attempts(&self) -> impl Iterator<Item = (&SetKey<G>, &Vec<Attempt>)> + '_ {
        self.attempts.iter().filter(|(k, _)| {
            !self
                .track_range
                .is_some_and(|(start, end)| k.location() < start || k.location() > end)
        })
    }

    fn push_attempt(&mut self, key: SetKey<G>, attempt: Attempt) {
        let location = key.location();
        if (attempt.duration() >= Duration::seconds_f64(2.0))
            && !self
                .track_range
                .is_some_and(|(start, end)| location < start || location > end)
        {
            self.attempts.entry(key).or_default().push(attempt);
        }
    }
}
