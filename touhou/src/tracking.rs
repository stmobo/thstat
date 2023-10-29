//! A framework for tracking events during the course of a game.
//!
//! This framework can be roughly divided into two halves:
//! - *Trackers,* which consume events such as game starts, game ends, location changes, and more.
//! - Game-specific *drivers* that interpret values read from an attached process's memory and convert them into events.
//!
//! The tracker half of this framework consists of traits which can be implemented by user types to define a tracker:
//! - The [`TrackGame`] trait defines the core of a tracker.
//! - The [`TrackRun`], [`TrackStagePractice`], and [`TrackSpellPractice`] subtraits define tracking logic for individual game types.
//! - The [`UpdateTracker`] trait defines logic for processing game events and location changes.
//!
//! The driver half, meanwhile, consists of the [`TrackerState`] type which encapsulates a tracker and maintains state necessary for
//! pushing events into it, and the [`DriveTracker`] trait which connects a driver type to its associated game memory reader type.
//!
//! Finally, there is the [`GameTracker`] type which can be used to connect a tracker to a driver;
//! it provides a simple interface to poll a driver for new events and retrieve the results of finished runs from the associated tracker.

use std::fmt::Display;
use std::hash::Hash;

use crate::memory::HasLocations;

pub mod tracker;

pub mod builder;

pub mod update;

pub mod state;

pub mod driver;

pub mod time;

#[doc(inline)]
pub use driver::GameTracker;
pub(crate) use driver::{DriveTracker, UpdateStatus};
#[doc(inline)]
pub use state::LocationResolveFilter;
#[doc(inline)]
pub use time::{EventTime, GameTimeCounter};
#[doc(inline)]
pub use tracker::{TrackGame, TrackRun, TrackSpellPractice, TrackStagePractice, UpdateTracker};

use crate::memory::Location;

/// Trait for games that can be used with this framework.
pub trait TrackableGame: HasLocations {
    type Event: std::fmt::Debug;
    type State: std::fmt::Debug;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event<G: TrackableGame> {
    Pause,
    Unpause,
    Miss,
    Bomb,
    Continue,
    GameSpecific(G::Event),
}

impl<G: TrackableGame> Event<G> {
    fn event_type_id(&self) -> u8 {
        match self {
            Self::Pause => 0,
            Self::Unpause => 1,
            Self::Miss => 2,
            Self::Bomb => 3,
            Self::Continue => 4,
            Self::GameSpecific(_) => 5,
        }
    }
}

impl<G> Clone for Event<G>
where
    G: TrackableGame,
    G::Event: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::GameSpecific(e) => Self::GameSpecific(e.clone()),
            Self::Miss => Self::Miss,
            Self::Bomb => Self::Bomb,
            Self::Pause => Self::Pause,
            Self::Unpause => Self::Unpause,
            Self::Continue => Self::Continue,
        }
    }
}

impl<G> Copy for Event<G>
where
    G: TrackableGame,
    G::Event: Copy,
{
}

impl<G> PartialEq for Event<G>
where
    G: TrackableGame,
    G::Event: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if let (Self::GameSpecific(a), Self::GameSpecific(b)) = (self, other) {
            a.eq(b)
        } else {
            self.event_type_id() == other.event_type_id()
        }
    }
}

impl<G> Eq for Event<G>
where
    G: TrackableGame,
    G::Event: Eq,
{
}

impl<G> PartialOrd for Event<G>
where
    G: TrackableGame,
    G::Event: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let (Self::GameSpecific(a), Self::GameSpecific(b)) = (self, other) {
            a.partial_cmp(b)
        } else {
            Some(self.event_type_id().cmp(&other.event_type_id()))
        }
    }
}

impl<G> Ord for Event<G>
where
    G: TrackableGame,
    G::Event: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let (Self::GameSpecific(a), Self::GameSpecific(b)) = (self, other) {
            a.cmp(b)
        } else {
            self.event_type_id().cmp(&other.event_type_id())
        }
    }
}

impl<G> Hash for Event<G>
where
    G: TrackableGame,
    G::Event: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.event_type_id().hash(state);
        if let Self::GameSpecific(data) = self {
            data.hash(state);
        }
    }
}

impl<G> Display for Event<G>
where
    G: TrackableGame,
    G::Event: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pause => "Pause".fmt(f),
            Self::Unpause => "Unpause".fmt(f),
            Self::Miss => "Miss".fmt(f),
            Self::Bomb => "Bomb".fmt(f),
            Self::Continue => "Continue".fmt(f),
            Self::GameSpecific(inner) => inner.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrackingType {
    FullRun,
    StagePractice,
    SpellPractice,
}

/// The state of a running game tracker.
///
/// This type handles keeping track of actual game values and detecting events using this information,
/// and can be created using a [`TrackerBuilder`](builder::TrackerBuilder).
///
/// Most user-facing code shouldn't need to use this type directly; game-specific driver code will handle
/// tracking game state for you.
#[derive(Debug)]
pub struct TrackerState<G: TrackableGame, T, L, B, C, P> {
    time: GameTimeCounter,
    location_filter: LocationResolveFilter<G>,
    track_type: TrackingType,
    tracker: T,
    lives: L,
    bombs: B,
    continues: C,
    pause: P,
}

impl<G: TrackableGame, T, L, B, C, P> TrackerState<G, T, L, B, C, P> {
    pub fn now(&self) -> EventTime {
        self.time.now()
    }

    pub fn start_time(&self) -> EventTime {
        self.time.start_time()
    }

    pub fn location(&self) -> Option<Location<G>> {
        self.location_filter.actual_location()
    }

    pub fn tracking_type(&self) -> TrackingType {
        self.track_type
    }
}
