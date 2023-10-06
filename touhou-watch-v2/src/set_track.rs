mod key;
mod metrics;
mod tracking;

pub use key::{MultiSetKey, SetKey};
pub use metrics::{Metrics, MetricsHandle};
pub use tracking::{ActiveGame, Attempt, SetTracker};

pub fn register_commands<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    metrics::register_commands(builder)
}
