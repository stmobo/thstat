/// Common wrappers and containers for game state extracted from running Touhou games.
use super::{GameLocation, HasLocations};

mod error;
mod location;
mod state;

#[doc(inline)]
pub use error::*;
#[doc(inline)]
pub use location::*;
#[doc(inline)]
pub use state::*;
