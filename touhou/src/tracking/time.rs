//! Types for keeping track of in-game time.

use std::time::{Duration, Instant, SystemTime};

/// Represents the time of a game event.
///
/// Event timestamps consist of:
/// - an [`Instant`],
/// - a corresponding [`SystemTime`],
/// - a [`Duration`] since the start of the associated game, and
/// - another [`Duration`] representing *play* time in the associated game (i.e. not including time spent paused).
///
/// New instances of this type can only be obtained via [`GameTimeCounter::now`].
///
/// The [`SystemTime`] and both [`Duration`]s contained in an event time are calculated based on the
/// associated [`Instant`], so all of them should be monotonic between event times measured using the same [`GameTimeCounter`] (barring platform bugs).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventTime {
    instant: Instant,      // used for duration computations
    timestamp: SystemTime, // actual timestamp of event
    game_time: Duration,   // time relative to start of game, including pause
    play_time: Duration,   // time relative to start of game, not counting time spent paused
}

impl EventTime {
    pub fn instant(&self) -> Instant {
        self.instant
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    pub fn game_time(&self) -> Duration {
        self.game_time
    }

    pub fn play_time(&self) -> Duration {
        self.play_time
    }

    pub fn time_between(&self, other: &EventTime) -> Duration {
        if self.instant >= other.instant {
            self.instant - other.instant
        } else {
            other.instant - self.instant
        }
    }

    pub fn play_time_between(&self, other: &EventTime) -> Duration {
        if self.play_time >= other.play_time {
            self.play_time - other.play_time
        } else {
            other.play_time - self.play_time
        }
    }
}

impl AsRef<Instant> for EventTime {
    fn as_ref(&self) -> &Instant {
        &self.instant
    }
}

impl AsRef<SystemTime> for EventTime {
    fn as_ref(&self) -> &SystemTime {
        &self.timestamp
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GameTimeCounter {
    game_start: (Instant, SystemTime),
    pause_counter: Duration,
    pause_start: Option<Instant>,
}

impl GameTimeCounter {
    /// Create a new time counter starting from the current time.
    pub fn new(paused: bool) -> Self {
        let game_start = (Instant::now(), SystemTime::now());
        let pause_start = if paused { Some(game_start.0) } else { None };
        Self {
            game_start,
            pause_start,
            pause_counter: Duration::ZERO,
        }
    }

    fn pause_duration_for(&self, instant: Instant) -> Duration {
        self.pause_counter
            + self
                .pause_start
                .map(|pause_start| instant.duration_since(pause_start))
                .unwrap_or(Duration::ZERO)
    }

    /// Get an [`EventTime`] representing when this counter started.
    ///
    /// The [`game_time`](method@EventTime::game_time) and [`play_time`](method@EventTime::play_time) of the
    /// returned event time are both guaranteed to be zero.
    ///
    /// # Example
    ///
    /// ```
    /// # use crate::tracking::GameTimeCounter;
    /// let counter = GameTimeCounter::new(false);
    /// let start_time = counter.start_time();
    ///
    /// assert_eq!(start_time.game_time(), Duration::ZERO);
    /// assert_eq!(start_time.play_time(), Duration::ZERO);
    /// ```
    pub fn start_time(&self) -> EventTime {
        let (instant, timestamp) = self.game_start;
        EventTime {
            instant,
            timestamp,
            game_time: Duration::ZERO,
            play_time: Duration::ZERO,
        }
    }

    /// Get the total accumulated time this counter has spent paused.
    pub fn total_pause_time(&self) -> Duration {
        self.pause_duration_for(Instant::now())
    }

    /// Get an [`EventTime`] representing the current time in-game.
    ///
    /// # Example
    ///
    /// ```
    /// # use touhou::tracking::GameTimeCounter;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// let counter = GameTimeCounter::new(false);
    /// let start_time = counter.start_time();
    /// sleep(Duration::from_millis(100));
    /// let now = counter.now();
    ///
    /// assert!(now.game_time() >= Duration::from_millis(100));
    /// assert_eq!(now.game_time(), now.instant() - start_time.instant());
    /// assert_eq!(
    ///     now.game_time(),
    ///     now.timestamp().duration_since(start_time.timestamp()).unwrap()
    /// );
    /// assert_eq!(now.play_time(), now.game_time()); // Counter wasn't paused during the sleep
    /// ```
    pub fn now(&self) -> EventTime {
        let instant = Instant::now();
        let game_time = instant.duration_since(self.game_start.0);
        let play_time = game_time - self.pause_duration_for(instant);
        let timestamp = self.game_start.1 + game_time;

        EventTime {
            instant,
            timestamp,
            game_time,
            play_time,
        }
    }

    /// Pauses the play time counter.
    ///
    /// # Example
    ///
    /// ```
    /// # use touhou::tracking::GameTimeCounter;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// let mut counter = GameTimeCounter::new(false); // Counter starts unpaused
    ///
    /// // Pause the counter and let it run
    /// counter.pause();
    /// sleep(Duration::from_millis(100));
    /// // Measure elapsed time
    /// let start_time = counter.start_time();
    /// let now = counter.now();
    ///
    /// // play_time doesn't count time spent paused, while game_time does:
    /// assert!(now.play_time() < Duration::from_millis(100));
    /// assert!(now.game_time() >= Duration::from_millis(100));
    /// ```
    pub fn pause(&mut self) {
        if self.pause_start.is_none() {
            self.pause_start = Some(Instant::now());
        }
    }

    /// Unpauses the play time counter.
    ///
    /// # Example
    ///
    /// ```
    /// # use touhou::tracking::GameTimeCounter;
    /// # use std::thread::sleep;
    /// # use std::time::Duration;
    /// let mut counter = GameTimeCounter::new(true); // Counter starts paused
    ///
    /// // Let it run paused for a bit, then unpause and let it run a bit more:
    /// sleep(Duration::from_millis(100));
    /// counter.unpause();
    /// sleep(Duration::from_millis(100));
    /// // Measure elapsed time
    /// let start_time = counter.start_time();
    /// let now = counter.now();
    ///
    /// // play_time doesn't count time spent paused, while game_time does:
    /// assert!(now.play_time() < Duration::from_millis(200));
    /// assert!(now.game_time() >= Duration::from_millis(200));
    pub fn unpause(&mut self) {
        if let Some(pause_start) = self.pause_start.take() {
            self.pause_counter += Instant::now().duration_since(pause_start);
        }
    }

    /// Update the pause state of this counter from a boolean indicating if the game is paused.
    ///
    /// This is just a convenience wrapper around the [`pause`] and [`unpause`] methods.
    pub fn update(&mut self, paused: bool) {
        match (self.pause_start.is_some(), paused) {
            (false, true) => self.pause(),
            (true, false) => self.unpause(),
            (false, false) | (true, true) => {}
        }
    }
}
