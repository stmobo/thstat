//! Traits for implementing trackers for Touhou games.

use super::{Event, EventTime, TrackableGame};
use crate::{Difficulty, Location, ShotType, Stage};

/// Supertrait for all types capable of tracking Touhou game events.
///
/// This trait defines the output of a tracker, as well as the type
/// (and thus logic) used to update it with new events and location info.
///
/// Types implementing this trait also need to implement this trait's
/// subtraits ([`TrackRun`], [`TrackStagePractice`], and [`TrackSpellPractice`])
/// depending on the game types they support.
pub trait TrackGame<G: TrackableGame> {
    /// The final output of this tracker once a game has ended.
    type Output;

    /// The type used to update this tracker with new events and locations.
    type Update<'a>: UpdateTracker<G>
    where
        Self: 'a;

    /// Begin updating this tracker with new state, events, and location info.
    fn begin_update(&mut self, time: EventTime, state: G::State) -> Self::Update<'_>;
}

/// Trait defining logic for processing game events and location changes.
pub trait UpdateTracker<G: TrackableGame> {
    /// Process a game event.
    fn push_event(&mut self, event: Event<G>);

    /// Process a change of location.
    fn change_location(&mut self, location: Option<Location<G>>);
}

/// Trait defining logic for tracking full game credits.
pub trait TrackRun<G: TrackableGame>: TrackGame<G> {
    /// Begin tracking a full game run.
    fn start_run(
        time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        state: G::State,
    ) -> Self;

    /// Finish tracking a successfully cleared run.
    fn run_cleared(self, time: EventTime, state: G::State) -> Self::Output;

    /// Finish tracking a run that has ended prematurely due to (for example) game over, retrying, or exiting the game.
    fn run_exited(self, time: EventTime, state: G::State) -> Self::Output;
}

/// Trait defining logic for tracking stage practice.
pub trait TrackStagePractice<G: TrackableGame>: TrackGame<G> {
    /// Begin tracking a stage practice attempt.
    fn start_stage_practice(
        time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        stage: Stage<G>,
        state: G::State,
    ) -> Self;

    /// Finish tracking a stage practice attempt.
    fn finish_stage_practice(self, time: EventTime, state: G::State) -> Self::Output;
}

/// Trait defining logic for tracking spell practice.
pub trait TrackSpellPractice<G: TrackableGame>: TrackGame<G> {
    /// Begin tracking a spell practice attempt.
    fn start_spell_practice(
        time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        state: G::State,
    ) -> Self;

    /// Finish tracking a spell practice attempt.
    fn finish_spell_practice(self, time: EventTime, state: G::State) -> Self::Output;
}
