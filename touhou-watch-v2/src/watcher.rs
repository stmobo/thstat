use std::fmt::Debug;
use std::io::Result as IOResult;
use std::thread::sleep;
use std::time::Duration;


use tauri::Window;
use touhou::memory::HasLocations;


use crate::event_serialize::AttachEvent;
use crate::set_track::{Metrics, SetTracker};


pub trait TrackedGame: Debug + HasLocations {
    type Reader: GameReader<Self>;

    fn autodetect_process() -> IOResult<Option<Self::Reader>>;
    fn get_tracker(metrics: &Metrics) -> &SetTracker<Self>;
    fn get_tracker_mut(metrics: &mut Metrics) -> &mut SetTracker<Self>;
}

pub trait GameReader<G: TrackedGame>: Debug + Sized {
    fn is_in_game(&mut self) -> IOResult<Option<bool>>;
    fn update(&mut self) -> IOResult<bool>;
    fn reset(&mut self);
    fn pid(&self) -> u32;
}

#[derive(Debug)]
struct Watcher<G: TrackedGame>(G::Reader);

impl<G: TrackedGame> Watcher<G> {
    fn wait_for_process(window: &Window) -> Self {
        loop {
            match G::autodetect_process() {
                Ok(Some(reader)) => {
                    window
                        .emit("attached", AttachEvent::from_reader::<G>(&reader))
                        .unwrap();

                    eprintln!(
                        "Attached to {}, PID {}",
                        G::GAME_ID.abbreviation(),
                        reader.pid()
                    );

                    return Self(reader);
                }
                Ok(None) => {}
                Err(e) => window.emit("error", e.to_string()).unwrap(),
            }

            sleep(Duration::from_secs(1));
        }
    }

    fn wait_for_game(&mut self) -> IOResult<bool> {
        loop {
            match self.0.is_in_game()? {
                Some(true) => {
                    sleep(Duration::from_secs(1));
                    match self.0.is_in_game()? {
                        Some(true) => return Ok(true),
                        Some(false) => continue,
                        None => return Ok(false),
                    }
                }
                Some(false) => {}
                None => return Ok(false),
            };

            sleep(Duration::from_millis(100));
        }
    }

    fn watch_game(&mut self, window: &Window) -> bool {
        self.0.reset();

        loop {
            match self.0.is_in_game() {
                Err(e) => window.emit("error", e.to_string()).unwrap(),
                Ok(Some(true)) => match self.0.update() {
                    Err(e) => window.emit("error", e.to_string()).unwrap(),
                    Ok(true) => window.emit("updated", G::GAME_ID).unwrap(),
                    Ok(false) => {}
                },
                Ok(Some(false)) => return true,
                Ok(None) => return false,
            }

            sleep(Duration::from_millis(10));
        }
    }

    fn watch_games(mut self, window: &Window) {
        let pid = self.0.pid();

        loop {
            window.emit("updated", G::GAME_ID).unwrap();
            match self.wait_for_game() {
                Err(e) => window.emit("error", e.to_string()).unwrap(),
                Ok(false) => break,
                Ok(true) => {
                    if !self.watch_game(window) {
                        break;
                    }

                    self.0.reset();
                }
            }
        }

        window.emit("updated", G::GAME_ID).unwrap();

        window
            .emit("detached", AttachEvent::new(G::GAME_ID, pid))
            .unwrap();
    }
}

pub fn track_game<G: TrackedGame>(window: Window) {
    loop {
        let watcher = Watcher::<G>::wait_for_process(&window);
        watcher.watch_games(&window);
    }
}
