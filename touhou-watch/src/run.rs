use std::collections::HashSet;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};
use time::error::IndeterminateOffset;
use time::OffsetDateTime;
use touhou::th07::memory::{GameState, PlayerState, StageLocation, StageSection, StageState};
use touhou::th07::{Difficulty, Stage, Touhou7};
use touhou::types::{ShotType, SpellCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EventTime(OffsetDateTime);

impl EventTime {
    pub fn new() -> Self {
        Self(OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()))
    }

    pub fn now_local() -> Result<Self, IndeterminateOffset> {
        OffsetDateTime::now_local().map(Self)
    }

    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}

impl Serialize for EventTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        let ts = self
            .unix_timestamp()
            .checked_mul(1000)
            .and_then(|x| x.checked_add(self.millisecond() as i64))
            .ok_or_else(|| S::Error::custom(format!("could not calculate timestamp in ms")))?;
        ts.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EventTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let ts = i64::deserialize(deserializer)?;
        let seconds = ts / 1000;
        let ms = ts % 1000;

        OffsetDateTime::from_unix_timestamp(seconds)
            .and_then(|t| t.replace_millisecond(ms as u16))
            .map(Self)
            .map_err(|e| D::Error::custom(e.to_string()))
    }
}

impl Default for EventTime {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for EventTime {
    type Target = OffsetDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EventTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<EventTime> for OffsetDateTime {
    fn from(value: EventTime) -> Self {
        value.0
    }
}

impl From<OffsetDateTime> for EventTime {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventKey(EventTime, u8);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    StartGame {
        time: EventTime,
        character: ShotType<Touhou7>,
        difficulty: Difficulty,
        location: StageLocation,
        practice: bool,
        lives: u8,
        bombs: u8,
        power: u8,
    },
    Pause {
        time: EventTime,
    },
    Unpause {
        time: EventTime,
    },
    EnterSection {
        time: EventTime,
        location: StageLocation,
        lives: u8,
        bombs: u8,
        power: u8,
        continues: u8,
    },
    Miss {
        time: EventTime,
        location: StageLocation,
    },
    Bomb {
        time: EventTime,
        location: StageLocation,
    },
    FinishSpell {
        time: EventTime,
        spell: SpellCard<Touhou7>,
        captured: bool,
    },
    BorderStart {
        time: EventTime,
        location: StageLocation,
    },
    BorderEnd {
        time: EventTime,
        location: StageLocation,
        broken: bool,
    },
    StageCleared {
        time: EventTime,
        stage: Stage,
    },
    EndGame {
        time: EventTime,
        location: StageLocation,
        misses: u32,
        bombs: u32,
        continues: u8,
        cleared: bool,
        retrying: bool,
    },
}

impl GameEvent {
    pub fn key(&self) -> EventKey {
        let type_key = match self {
            Self::StartGame { .. } => 0,
            Self::Pause { .. } => 1,
            Self::Unpause { .. } => 2,
            Self::EnterSection { .. } => 3,
            Self::Miss { .. } => 4,
            Self::Bomb { .. } => 5,
            Self::FinishSpell { .. } => 6,
            Self::BorderStart { .. } => 7,
            Self::BorderEnd { .. } => 8,
            Self::StageCleared { .. } => 9,
            Self::EndGame { .. } => 10,
        };

        EventKey(self.time(), type_key)
    }

    pub fn time(&self) -> EventTime {
        match self {
            Self::StartGame { time, .. }
            | Self::EndGame { time, .. }
            | Self::EnterSection { time, .. }
            | Self::StageCleared { time, .. }
            | Self::Miss { time, .. }
            | Self::Bomb { time, .. }
            | Self::FinishSpell { time, .. }
            | Self::BorderStart { time, .. }
            | Self::BorderEnd { time, .. }
            | Self::Pause { time, .. }
            | Self::Unpause { time, .. } => *time,
        }
    }
}

impl Display for GameEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartGame {
                character,
                location,
                practice,
                difficulty,
                ..
            } => {
                if *practice {
                    write!(
                        f,
                        "Started {} practice game as {} at {}",
                        difficulty, character, location
                    )
                } else {
                    write!(
                        f,
                        "Started {} game as {} at {}",
                        difficulty, character, location
                    )
                }
            }
            Self::EndGame {
                location,
                misses,
                bombs,
                continues,
                cleared,
                retrying,
                ..
            } => {
                write!(
                    f,
                    "{} game at {} with {} miss{}, {} bomb{}, and {} continue{} used",
                    if *cleared {
                        "Cleared"
                    } else if *retrying {
                        "Retried"
                    } else {
                        "Ended"
                    },
                    location,
                    *misses,
                    if *misses != 1 { "es" } else { "" },
                    *bombs,
                    if *bombs != 1 { "s" } else { "" },
                    *continues,
                    if *continues != 1 { "s" } else { "" }
                )
            }
            Self::EnterSection {
                location,
                lives,
                bombs,
                power,
                continues,
                ..
            } => {
                write!(
                    f,
                    "Entering {} with {} {}, {} bomb{}, {} power, and {} continue{} used",
                    location,
                    *lives,
                    if *lives == 1 { "life" } else { "lives" },
                    *bombs,
                    if *bombs != 1 { "s" } else { "" },
                    *power,
                    *continues,
                    if *continues != 1 { "s" } else { "" }
                )
            }
            Self::StageCleared { stage, .. } => write!(f, "Cleared {}", stage),
            Self::Miss { location, .. } => write!(f, "Missed at {}", location),
            Self::Bomb { location, .. } => write!(f, "Bombed at {}", location),
            Self::FinishSpell {
                spell,
                captured: capture,
                ..
            } => {
                write!(
                    f,
                    "{} #{:03} {}",
                    if *capture { "Captured" } else { "Failed" },
                    spell.id(),
                    spell.name()
                )
            }
            Self::BorderStart { location, .. } => {
                write!(f, "Border started at {}", location)
            }
            Self::BorderEnd {
                broken, location, ..
            } => {
                if *broken {
                    write!(f, "Border broken at {}", location)
                } else {
                    write!(f, "Border ended at {}", location)
                }
            }
            Self::Pause { .. } => f.write_str("Paused game"),
            Self::Unpause { .. } => f.write_str("Unpaused game"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    start_time: EventTime,
    end_info: Option<(bool, EventTime)>,
    location: StageLocation,
    practice: bool,
    shot: ShotType<Touhou7>,
    difficulty: Difficulty,
    misses: Vec<(EventTime, StageLocation)>,
    bombs: Vec<(EventTime, StageLocation)>,
    breaks: Vec<(EventTime, StageLocation)>,
    locations_seen: HashSet<StageLocation>,
    score: u32,
    continues: u8,
    #[serde(deserialize_with = "Run::deserialize_sorted_events")]
    events: Vec<GameEvent>,
}

impl Run {
    fn push_event(&mut self, event: GameEvent) {
        let index = self
            .events
            .binary_search_by_key(&event.key(), |ev| ev.key())
            .unwrap_or_else(|idx| idx);
        self.events.insert(index, event);
    }

    fn deserialize_sorted_events<'de, D>(deserializer: D) -> Result<Vec<GameEvent>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut events = <Vec<GameEvent> as Deserialize>::deserialize(deserializer)?;
        events.sort_by_key(|ev| ev.key());
        Ok(events)
    }
}

#[derive(Debug, Clone)]
pub enum UpdateResult {
    Continuing(ActiveRun),
    Finished(Run, usize),
}

impl UpdateResult {
    pub fn new_events(&self) -> &[GameEvent] {
        match self {
            UpdateResult::Continuing(run) => run.new_events(),
            UpdateResult::Finished(run, prev_update_events) => &run.events[*prev_update_events..],
        }
    }

    pub fn run(&self) -> &Run {
        match self {
            UpdateResult::Continuing(run) => run.run(),
            UpdateResult::Finished(run, _) => run,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished(_, _))
    }
}

#[derive(Debug, Clone)]
pub struct ActiveRun {
    run: Run,
    update_time: EventTime,
    last_border_start: Option<EventTime>,
    paused: bool,
    player_state: PlayerState,
    stage_state: StageState,
    prev_update_events: usize,
}

impl ActiveRun {
    pub fn new(
        practice: bool,
        paused: bool,
        player: PlayerState,
        stage: StageState,
    ) -> Option<Self> {
        let location = stage.location()?;
        let mut locations_seen = HashSet::with_capacity(1);
        locations_seen.insert(location);

        let start_time = EventTime::new();

        let init_event = GameEvent::StartGame {
            time: start_time,
            character: player.character(),
            difficulty: player.difficulty(),
            location,
            practice,
            lives: player.lives(),
            bombs: player.bombs(),
            power: player.power(),
        };

        let run = Run {
            start_time,
            end_info: None,
            location,
            locations_seen,
            practice,
            shot: player.character(),
            difficulty: player.difficulty(),
            misses: Vec::new(),
            bombs: Vec::new(),
            breaks: Vec::new(),
            score: player.score(),
            continues: player.continues(),
            events: vec![init_event],
        };

        let mut ret = Self {
            run,
            update_time: EventTime::new(),
            last_border_start: None,
            paused: false,
            player_state: player,
            stage_state: stage,
            prev_update_events: 0,
        };

        if player.border_active() {
            ret.update_border(true);
        }

        if paused {
            ret.update_pause_state(true);
        }

        Some(ret)
    }

    pub fn current_location(&self) -> StageLocation {
        self.run.location
    }

    fn events(&self) -> &Vec<GameEvent> {
        &self.run.events
    }

    fn push_event(&mut self, event: GameEvent) {
        self.run.push_event(event);
    }

    fn boss_finished(&self) -> bool {
        self.current_location().is_end_spell()
            && self
                .stage_state
                .boss_state()
                .map(|boss| boss.active_spell().is_none())
                .unwrap_or(true)
    }

    fn update_pause_state(&mut self, new_paused: bool) {
        match (self.paused, new_paused) {
            (false, true) => self.push_event(GameEvent::Pause {
                time: self.update_time,
            }),
            (true, false) => self.push_event(GameEvent::Unpause {
                time: self.update_time,
            }),
            _ => {}
        }

        self.paused = new_paused;
    }

    fn update_border(&mut self, border_active: bool) {
        if border_active {
            if self.last_border_start.is_none() {
                self.last_border_start = Some(self.update_time);
                self.push_event(GameEvent::BorderStart {
                    time: self.update_time,
                    location: self.current_location(),
                });
            }
        } else if let Some(duration) = self.last_border_start.take().and_then(|start| {
            (self.update_time.0 - start.0)
                .max(time::Duration::ZERO)
                .try_into()
                .ok()
        }) {
            // don't treat border as broken if it happens at end of stage or before a boss fight
            let duration: Duration = duration;
            let broken = (duration <= Duration::from_millis(8925))
                && (self.current_location().section() != StageSection::PreBoss)
                && !self.boss_finished();

            if broken {
                self.run
                    .breaks
                    .push((self.update_time, self.current_location()));
            }

            self.push_event(GameEvent::BorderEnd {
                time: self.update_time,
                broken,
                location: self.current_location(),
            });
        }
    }

    fn finish_spell(&mut self, spell: SpellCard<Touhou7>, captured: bool) {
        self.push_event(GameEvent::FinishSpell {
            time: self.update_time,
            spell,
            captured,
        });

        if self.boss_finished() {
            self.push_event(GameEvent::StageCleared {
                time: self.update_time,
                stage: self.current_location().stage(),
            });
        }
    }

    fn update_state(&mut self, stage_state: StageState, player_state: PlayerState) {
        self.update_time = EventTime::new();
        self.prev_update_events = self.events().len();

        let prev_player_state = std::mem::replace(&mut self.player_state, player_state);
        let prev_stage_state = std::mem::replace(&mut self.stage_state, stage_state);

        if let Some(location) = stage_state.location() {
            if location != self.run.location {
                self.run.locations_seen.insert(location);
                self.push_event(GameEvent::EnterSection {
                    time: self.update_time,
                    location,
                    lives: player_state.lives(),
                    bombs: player_state.bombs(),
                    power: player_state.power(),
                    continues: player_state.continues(),
                })
            }

            self.run.location = location;
        }

        if player_state.border_active() != prev_player_state.border_active() {
            self.update_border(player_state.border_active());
        }

        if player_state.total_bombs() == (prev_player_state.total_bombs() + 1) {
            self.run
                .bombs
                .push((self.update_time, self.current_location()));

            self.push_event(GameEvent::Bomb {
                time: self.update_time,
                location: self.current_location(),
            });
        }

        if player_state.total_misses() == (prev_player_state.total_misses() + 1) {
            self.run
                .misses
                .push((self.update_time, self.current_location()));

            self.push_event(GameEvent::Miss {
                time: self.update_time,
                location: self.current_location(),
            });
        }

        let cur_boss_spell = stage_state
            .boss_state()
            .as_ref()
            .and_then(|x| x.active_spell())
            .map(|x| (SpellCard::new((x.0 + 1).try_into().unwrap()), x.1));

        let prev_boss_spell = prev_stage_state
            .boss_state()
            .as_ref()
            .and_then(|x| x.active_spell())
            .map(|x| (SpellCard::new((x.0 + 1).try_into().unwrap()), x.1));

        match (prev_boss_spell, cur_boss_spell) {
            (Some(prev), Some(cur)) => {
                if prev.0 != cur.0 {
                    self.finish_spell(prev.0, prev.1);
                }
            }
            (Some(prev), None) => self.finish_spell(prev.0, prev.1),
            _ => {}
        }

        self.player_state = player_state;
        self.stage_state = stage_state;
    }

    pub fn end_game(
        mut self,
        mut cleared: bool,
        retrying: bool,
        end_states: Option<(PlayerState, StageState)>,
    ) -> (Run, usize) {
        if let Some((player_state, stage_state)) = end_states {
            self.update_state(stage_state, player_state);
        } else {
            self.update_time = EventTime::new();
            self.prev_update_events = self.events().len();
        }

        self.update_border(false);
        if self.run.practice && self.boss_finished() {
            cleared = true;
        }

        self.push_event(GameEvent::EndGame {
            time: self.update_time,
            location: self.current_location(),
            misses: self.player_state.total_misses(),
            bombs: self.player_state.total_bombs(),
            continues: self.player_state.continues(),
            retrying,
            cleared,
        });

        self.run.end_info = Some((cleared, self.update_time));

        (self.run, self.prev_update_events)
    }

    pub fn update(mut self, state: GameState) -> UpdateResult {
        match state {
            GameState::InGame {
                paused,
                stage,
                player,
                ..
            } => {
                self.update_state(stage, player);
                self.update_pause_state(paused);
                UpdateResult::Continuing(self)
            }
            GameState::LoadingStage => UpdateResult::Continuing(self),
            GameState::GameOver {
                cleared,
                player,
                stage,
            } => {
                let (run, prev_update_events) =
                    self.end_game(cleared, false, Some((player, stage)));
                UpdateResult::Finished(run, prev_update_events)
            }
            GameState::RetryingGame => {
                let (run, prev_update_events) = self.end_game(false, true, None);
                UpdateResult::Finished(run, prev_update_events)
            }
            GameState::Unknown { state_id, mode } => {
                eprintln!("observed unknown state {}/{}", state_id, mode);
                UpdateResult::Continuing(self)
            }
            _ => {
                let (run, prev_update_events) = self.end_game(false, false, None);
                UpdateResult::Finished(run, prev_update_events)
            }
        }
    }

    pub fn new_events(&self) -> &[GameEvent] {
        &self.events()[self.prev_update_events..]
    }

    pub fn run(&self) -> &Run {
        &self.run
    }
}
