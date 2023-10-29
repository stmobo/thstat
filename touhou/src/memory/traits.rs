//! Traits representing the different kinds of game state that can be extracted from running Touhou processes.

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::types::SpellState;
use crate::types::Game;
use crate::{Difficulty, Location, ShotPower, ShotType, SpellCard, Stage};

/// Top-level trait for accessing the state of an active Touhou game.
///
/// Types that implement this trait represent snapshots of in-progress
/// games of Touhou, both stage practice and regular credits.
///
/// Using the methods on this type, you can dig further into more specific
/// state regarding both [the player](PlayerData) and the [current stage](StageData) they're playing,
/// as well as general information about the run such as the difficulty setting, whether the game is paused
/// (via the optional [`PauseState`] trait), and so on.
///
/// Additional traits that might be implemented on types with this trait currently include:
/// - [`PauseState`]
/// - [`ResolveLocation`]
pub trait RunData<G: Game>: Sized {
    type StageState: StageData<G>;
    type PlayerState: PlayerData<G>;

    fn difficulty(&self) -> Difficulty<G>;
    fn player(&self) -> &Self::PlayerState;
    fn stage(&self) -> &Self::StageState;
    fn is_practice(&self) -> bool;
}

/// Trait for checking whether or not a Touhou game is currently paused.
///
/// This is implemented alongside [`RunData`] for games with memory readers that can
/// determine whether the game is paused or not. This currently includes PCB and IN, but not MoF.
pub trait PauseState {
    fn paused(&self) -> bool;
}

/// Trait for accessing data about the current stage being played in an active Touhou game.
///
/// Types that implement this trait contain specific information about the stage (such as its ID)
/// and about the enemies within it. In particular, methods on this trait can be used to access
/// information about any bosses currently being fought, such as whether or not a spell card is active.
///
/// Many of the other traits that are implemented alongside this trait and the [`BossData`] trait
/// are useful for resolving the player's current location within the game.
///
/// Additional traits that might be implemented on types with this trait currently include:
/// - [`ECLTimeline`]
pub trait StageData<G: Game>: Sized {
    type BossState: BossData<G>;

    fn stage_id(&self) -> Stage<G>;
    fn active_boss(&self) -> Option<&Self::BossState>;

    fn active_spell(&self) -> Option<SpellState<G>> {
        self.active_boss().and_then(|boss| boss.active_spell())
    }
}

/// Trait for accessing the first-generation Windows engine's ECL timeline state.
///
/// This trait is implemented alongside [`StageData`] for the first-generation Windows games,
/// which use a timeline-based system for running ECL. The [`ecl_time`](`ECLTimeline::ecl_time`)
/// method returns the player's current position within the timeline, which can be used to identify subsections of stages.
pub trait ECLTimeline<G: Game>: StageData<G> {
    fn ecl_time(&self) -> u32;
}

/// Trait for accessing data about a active boss fight in a Touhou game.
///
/// Types that implement this trait contain information about boss fights,
/// which at minimum includes whether or not any of their spell cards are active.
///
/// Additional traits that might be implemented on types with this trait currently include:
/// - [`BossLifebars`]
pub trait BossData<G: Game>: Sized {
    fn active_spell(&self) -> Option<SpellState<G>>;
}

/// Trait for checking how many life bars a boss has left.
///
/// This is implemented alongside [`BossData`] for games with memory
/// readers that can extract how many extra life bars a boss has left
/// (which in-game is typically represented by a number of hearts or
/// stars in the top corners of the screen).
///
/// This information is useful for distinguishing individual nonspells
/// within a boss fight when resolving the player's current location.
pub trait BossLifebars<G: Game>: BossData<G> {
    fn remaining_lifebars(&self) -> u8;
}

/// Trait for accessing data about the player within an active Touhou game.
///
/// The methods on this trait encompass state and game mechanics that are
/// universal across the franchise, which at this point only consists of
/// shot type and shot power.
///
/// Additional traits can be implemented alongside this trait to expose information
/// about less universal mechanics:
/// - [`LifeStock`]
/// - [`MissCount`]
/// - [`BombStock`]
/// - [`BombCount`]
/// - [`PlayerScore`]
/// - [`ContinueCount`]
pub trait PlayerData<G: Game>: Sized {
    fn shot(&self) -> ShotType<G>;
    fn power(&self) -> ShotPower<G>;
}

/// Trait for accessing the player's current stock of lives within a Touhou game.
pub trait LifeStock<G: Game>: PlayerData<G> + Sized {
    fn lives(&self) -> u8;
}

/// Trait for accessing the player's total miss count over the course of a Touhou game.
///
/// As far as I am aware, this information is only tracked by the first-generation Windows
/// games for the end-game statistics display, and by LoLK's Pointdevice mode in the form
/// of the retry counter.
pub trait MissCount<G: Game>: PlayerData<G> + Sized {
    fn total_misses(&self) -> u8;
}

/// Trait for accessing the player's current stock of bombs, for Touhou games that do item-based bombing.
pub trait BombStock<G: Game>: PlayerData<G> + Sized {
    fn bombs(&self) -> u8;
}

/// Trait for accessing the player's total count of bombs used over the course of a Touhou game.
///
/// As far as I am aware, this is only tracked by the first-generation Windows games for the end-game
/// statistics display.
pub trait BombCount<G: Game>: BombStock<G> + Sized {
    fn total_bombs(&self) -> u8;
}

/// Trait for counting how many continues the player has used, for games that track this information.
pub trait ContinueCount<G: Game>: PlayerData<G> + Sized {
    fn continues_used(&self) -> u8;
}

/// Trait for getting the player's current score.
pub trait PlayerScore<G: Game>: PlayerData<G> + Sized {
    fn score(&self) -> u64;
}

/// Trait for statelessly finding where the player currently is in an active Touhou game.
///
/// This is generally implemented alongside [`RunData`] for games that support
/// resolving location information directly from run states.
pub trait ResolveLocation<G: HasLocations> {
    /// Attempt to resolve the player's current location based on gathered runtime state.
    ///
    /// This method should return `None` if the player's location is indeterminate or otherwise invalid.
    fn resolve_location(&self) -> Option<Location<G>>;
}

/// Trait for stateful location resolution from Touhou game states.
///
/// Some games can't accurately determine player location without knowledge of previous game state,
/// and thus must implement this trait instead of [`ResolveLocation`].
pub trait TrackLocation<G: HasLocations, S> {
    fn update_location(&mut self, state: S) -> Option<Location<G>>;
}

/// Trait for game-specific types representing in-game locations.
///
/// This trait is implemented by game-specific location types such as [`crate::th07::Location`] and [`crate::th10::Location`].
///
/// In general, the [`Location`](`super::types::Location`) wrapper type should be more convenient to use than
/// the underlying game-specific location types.
pub trait GameLocation<G: Game>:
    std::fmt::Debug + Copy + Eq + Ord + std::hash::Hash + Default + Serialize + DeserializeOwned
{
    fn name(&self) -> &'static str;
    fn index(&self) -> u64;
    fn stage(&self) -> Stage<G>;
    fn spell(&self) -> Option<SpellCard<G>>;
    fn is_end(&self) -> bool;
    fn is_boss_start(&self) -> bool;
    fn from_spell(spell: SpellCard<G>) -> Option<Self>;
}

/// Trait for games that have defined location information.
///
/// Actually getting the player's location in such a game is done via the [`ResolveLocation`] trait,
/// not this trait.
pub trait HasLocations: Game {
    type Location: GameLocation<Self>;

    fn stage_start_location(stage: Self::StageID) -> Self::Location;
}
