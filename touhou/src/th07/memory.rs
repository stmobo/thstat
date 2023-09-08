pub mod location;
pub mod process;
pub mod state;

pub use location::{StageLocation, StageSection};
pub use process::GameMemory;
pub use state::{BossState, GameState, PlayerState, StageState};
