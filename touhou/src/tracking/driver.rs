//! Logic for driving game trackers.

use std::marker::PhantomData;
use std::time::{Duration, Instant};

use super::{TrackGame, TrackableGame};
use crate::memory::{GameMemory, MemoryReadError};

#[derive(Debug)]
pub enum UpdateStatus<G: TrackableGame, T: TrackGame<G>, D: DriveTracker<G, T>> {
    Continuing(D),
    Finished(T::Output),
}

/// Game-specific logic for driving a [`GameTracker`].
///
/// This trait serves as a bridge between a game-specific memory reader type (implementing [`GameMemory`])
/// and types implementing [`TrackGame`] and related interfaces.
///
/// User-facing code shouldn't need to work with this trait directly, and should instead use [`GameTracker`].
pub trait DriveTracker<G: TrackableGame, T: TrackGame<G>>: Sized {
    /// The game-specific memory reader type used by this driver.
    type Memory: GameMemory<G>;

    /// Detect whether a game is currently active in the attached process.
    ///
    /// In general, this method should return `true` whenever [`update`](DriveTracker::update) can be called,
    /// which can include states not normally associated with 'active' games, such as states for games
    /// that have just concluded, games that are being retried, and load screens in-between stages.
    ///
    /// This method is used to detect the very start of a new game, and as such should be prepared
    /// to potentially race against the attached game process initializing its own internal state.
    fn game_is_active(
        access: &<Self::Memory as GameMemory<G>>::MemoryAccess,
    ) -> Result<bool, MemoryReadError<G>>;

    /// Initialize a new driver instance for an active game.
    ///
    /// This method is called one second after a game is first detected via [`game_is_active`](DriveTracker::game_is_active),
    /// and as such can (generally) assume that the game state is fully initialized.
    fn init(
        access: &<Self::Memory as GameMemory<G>>::MemoryAccess,
    ) -> Result<Option<Self>, MemoryReadError<G>>;

    /// Update this driver instance using memory values read from the attached process.
    ///
    /// This method should either return itself wrapped inside of [`UpdateStatus::Continuing`] if
    /// the active game is ongoing, and the tracker's output within [`UpdateStatus::Finished`] once it has finished.
    fn update(
        self,
        access: &<Self::Memory as GameMemory<G>>::MemoryAccess,
    ) -> Result<UpdateStatus<G, T, Self>, MemoryReadError<G>>;

    /// Terminate tracking for the current game.
    ///
    /// This is called if the attached process exits, or if user code calls [`close`](GameTracker::close) mid-game.
    fn terminate(self) -> T::Output;
}

#[derive(Debug)]
enum GameInitState<G: TrackableGame, T: TrackGame<G>, D: DriveTracker<G, T>> {
    Updating,
    WaitingForGame,
    WaitingForInit(Instant),
    Active(D, PhantomData<(G, T)>),
}

/// Tracks game events within a running Touhou process.
///
/// This is the main interface to the game-tracking framework for user code,
/// and does all the necessary work of detecting active games, creating tracker instances,
/// feeding them new state and events, and collecting the results after each run.
///
/// You can instantiate this type using methods on the game memory readers for supported games
/// (such as [`track_games`](crate::th07::GameMemory::track_games) for Touhou 7), or by using
/// [`GameTracker::new`].
///
/// Once instantiated, the tracker can be polled for updates using the [`update`](GameTracker::update) method,
/// which will return tracker outputs for games as they are completed.
/// Once the attached process has exited (which can be detected through the [`is_running`](GameTracker::is_running) method),
/// you can call [`close`](GameTracker::close) to collect the results of any interrupted games.
///
/// Note that the tracker update logic adds a 1-second delay from when a new game is first detected before starting
/// to track it; this is to ensure that the game process has time to properly initialize its internal state
/// before we begin reading values.
#[derive(Debug)]
pub struct GameTracker<G: TrackableGame, T: TrackGame<G>, D: DriveTracker<G, T>> {
    state: GameInitState<G, T, D>,
    memory: D::Memory,
}

impl<G: TrackableGame, T: TrackGame<G>, D: DriveTracker<G, T>> GameTracker<G, T, D> {
    /// Create a new tracker wrapping the given game memory type.
    pub fn new(memory: D::Memory) -> Self {
        Self {
            memory,
            state: GameInitState::WaitingForGame,
        }
    }

    /// Get a reference to the contained game memory instance.
    pub fn memory(&self) -> &D::Memory {
        &self.memory
    }

    /// Get a mutable reference to the contained game memory instance.
    pub fn memory_mut(&mut self) -> &mut D::Memory {
        &mut self.memory
    }

    /// Get the PID of this tracker's attached process.
    pub fn pid(&self) -> u32 {
        self.memory.pid()
    }

    /// Get whether this tracker's attached process is still running.
    pub fn is_running(&mut self) -> bool {
        self.memory.is_running()
    }

    /// Update the tracker by reading new values from the attached game process.
    ///
    /// If a game has been completed, this method will return the tracker's output.
    pub fn update(&mut self) -> Result<Option<T::Output>, MemoryReadError<G>> {
        if let Some(access) = self.memory.access() {
            if D::game_is_active(access)? {
                return match std::mem::replace(&mut self.state, GameInitState::Updating) {
                    GameInitState::WaitingForGame => {
                        self.state = GameInitState::WaitingForInit(Instant::now());
                        Ok(None)
                    }
                    GameInitState::WaitingForInit(start) => {
                        self.state = GameInitState::WaitingForInit(start);
                        if Instant::now().duration_since(start) >= Duration::from_millis(1000) {
                            if let Some(driver) = D::init(access)? {
                                self.state = GameInitState::Active(driver, PhantomData)
                            }
                        }
                        Ok(None)
                    }
                    GameInitState::Active(driver, _) => match driver.update(access)? {
                        UpdateStatus::Continuing(driver) => {
                            self.state = GameInitState::Active(driver, PhantomData);
                            Ok(None)
                        }
                        UpdateStatus::Finished(output) => {
                            self.state = GameInitState::WaitingForGame;
                            Ok(Some(output))
                        }
                    },
                    GameInitState::Updating => unreachable!(), // shouldn't happen
                };
            }
        }

        if let GameInitState::Active(driver, _) =
            std::mem::replace(&mut self.state, GameInitState::WaitingForGame)
        {
            Ok(Some(driver.terminate()))
        } else {
            Ok(None)
        }
    }

    /// Close this tracker, terminating tracking for any games currently in progress.
    ///
    /// Returns the contained memory instance, as well as tracker output for the current
    /// game if one was in progress.
    pub fn close(self) -> (D::Memory, Option<T::Output>) {
        if let GameInitState::Active(driver, _) = self.state {
            (self.memory, Some(driver.terminate()))
        } else {
            (self.memory, None)
        }
    }
}

/// A convenience trait for getting a [`GameTracker`] from a game memory reader.
pub trait IntoGameTracker<G: TrackableGame, T: TrackGame<G>> {
    type Driver: DriveTracker<G, T>;

    fn track_games(self) -> GameTracker<G, T, Self::Driver>;
}
