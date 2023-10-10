use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};
use time::error::IndeterminateOffset;
use time::{Duration, OffsetDateTime};
use touhou::memory::PauseState;

mod serde_conv;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[repr(transparent)]
#[serde(try_from = "serde_conv::Milliseconds")]
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
        use serde_conv::Milliseconds;
        Milliseconds::try_from(*self)
            .map_err(S::Error::custom)
            .and_then(move |ms| ms.serialize(serializer))
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameTime {
    timestamp: EventTime,
    #[serde(with = "serde_conv::duration")]
    relative_real_time: Duration, // full time since game start, includes pauses
    #[serde(with = "serde_conv::duration")]
    relative_game_time: Duration, // playtime since game start, not including pauses
}

impl GameTime {
    pub fn game_duration_between(&self, other: &GameTime) -> Duration {
        if self.relative_game_time < other.relative_game_time {
            other.game_duration_between(self)
        } else {
            self.relative_game_time - other.relative_game_time
        }
    } 
}

#[derive(Debug, Clone, Copy)]
pub struct GameTimeCounter {
    start_time: EventTime,
    total_pause_time: Duration,
    last_pause: Option<EventTime>,
}

impl GameTimeCounter {
    pub fn new(start_time: EventTime) -> Self {
        Self {
            start_time,
            total_pause_time: Duration::ZERO,
            last_pause: None,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.last_pause.is_some()
    }

    pub fn pause(&mut self) {
        if self.last_pause.is_none() {
            self.last_pause = Some(EventTime::new());
        }
    }

    pub fn unpause(&mut self) {
        if let Some(pause_time) = self.last_pause.take() {
            let now = EventTime::new();
            self.total_pause_time += *now - *pause_time;
        }
    }

    pub fn update_from_state<T: PauseState>(&mut self, state: &T) {
        match (self.is_paused(), state.paused()) {
            (false, true) => self.pause(),
            (true, false) => self.unpause(),
            (true, true) | (false, false) => {}
        }
    }

    pub fn now(&self) -> GameTime {
        let timestamp = EventTime::new();
        let relative_real_time = *timestamp - *self.start_time;
        GameTime {
            timestamp,
            relative_real_time,
            relative_game_time: (relative_real_time - self.total_pause_time).max(Duration::ZERO),
        }
    }
}

impl Default for GameTimeCounter {
    fn default() -> Self {
        Self::new(EventTime::new())
    }
}
