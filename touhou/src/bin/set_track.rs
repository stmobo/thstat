use std::fmt::Display;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use touhou::memory::{GameMemory, MemoryReadError};
use touhou::th07::{GameMemory as Th07Memory, Touhou7Event};
use touhou::th10::GameMemory as Th10Memory;
use touhou::tracking::{
    Event, EventTime, GameTracker, IntoGameTracker, TrackGame, TrackRun, TrackSpellPractice,
    TrackStagePractice, TrackableGame, UpdateTracker,
};
use touhou::{Difficulty, HasLocations, Location, ShotType, Touhou10, Touhou7};

#[derive(Debug, Clone, Copy)]
struct DisplayTime(SystemTime);

impl Default for DisplayTime {
    fn default() -> Self {
        Self(SystemTime::now())
    }
}

impl Display for DisplayTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = match self.0.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_secs_f64(),
            Err(e) => -e.duration().as_secs_f64(),
        };

        write!(f, "{secs:.3}")
    }
}

#[derive(Debug, Clone, Copy)]
struct DisplayPlayTime(Duration);

impl Display for DisplayPlayTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.0.as_secs_f64();
        write!(f, "{secs:+.3}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SetKey<G: HasLocations> {
    difficulty: Difficulty<G>,
    shot: ShotType<G>,
    location: Location<G>,
}

impl<G: HasLocations> Display for SetKey<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}, {})", self.location, self.shot, self.difficulty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Attempt {
    start_time: EventTime,
    end_time: EventTime,
    success: bool,
}

impl Display for Attempt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let play_time = self.end_time.play_time_between(&self.start_time);
        write!(
            f,
            "{} ({:.2}s)",
            if self.success { "Success" } else { "Failure" },
            play_time.as_secs_f64()
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct ActiveSetInfo<G: HasLocations> {
    start_time: EventTime,
    location: Location<G>,
    success: bool,
}

#[derive(Debug)]
struct SetTracker<G: HasLocations> {
    shot: ShotType<G>,
    difficulty: Difficulty<G>,
    attempts: Vec<(SetKey<G>, Attempt)>,
    current: Option<ActiveSetInfo<G>>,
}

impl<G: HasLocations> SetTracker<G> {
    fn finish(self, now: EventTime, cleared: bool) -> Vec<(SetKey<G>, Attempt)> {
        let Self {
            shot,
            difficulty,
            mut attempts,
            current,
        } = self;

        if let Some(current) = current {
            let key = SetKey {
                difficulty,
                shot,
                location: current.location,
            };
            let attempt = Attempt {
                start_time: current.start_time,
                end_time: now,
                success: current.success && cleared,
            };
            attempts.push((key, attempt));
        }

        attempts
    }
}

impl<G> TrackGame<G> for SetTracker<G>
where
    G: TrackableGame,
    for<'a> SetTrackerUpdate<'a, G>: UpdateTracker<G>,
{
    type Output = Vec<(SetKey<G>, Attempt)>;
    type Update<'a> = SetTrackerUpdate<'a, G>
    where
        Self: 'a;

    fn begin_update(
        &mut self,
        time: EventTime,
        _state: <G as TrackableGame>::State,
    ) -> Self::Update<'_> {
        SetTrackerUpdate {
            tracker: self,
            now: time,
        }
    }
}

#[derive(Debug)]
struct SetTrackerUpdate<'a, G: HasLocations> {
    tracker: &'a mut SetTracker<G>,
    now: EventTime,
}

impl<'a, G: HasLocations> SetTrackerUpdate<'a, G> {
    fn push_attempt(&mut self, new_info: Option<ActiveSetInfo<G>>) {
        if let Some(prev) = std::mem::replace(&mut self.tracker.current, new_info) {
            let key = SetKey {
                difficulty: self.tracker.difficulty,
                shot: self.tracker.shot,
                location: prev.location,
            };

            let attempt = Attempt {
                start_time: prev.start_time,
                end_time: self.now,
                success: prev.success,
            };

            println!("[{0}]     {key}: {attempt}", DisplayTime::default());

            self.tracker.attempts.push((key, attempt));
        }
    }
}

impl<'a> UpdateTracker<Touhou7> for SetTrackerUpdate<'a, Touhou7> {
    fn push_event(&mut self, event: Event<Touhou7>) {
        if let Some(ref current) = self.tracker.current {
            println!(
                "[{0}] {event} at {1}",
                DisplayTime::default(),
                current.location
            );
        } else {
            println!("[{0}] {event}", DisplayTime::default());
        }

        match event {
            Event::Miss
            | Event::Bomb
            | Event::Continue
            | Event::GameSpecific(Touhou7Event::BorderEnd { broken: true }) => {
                if let Some(current) = &mut self.tracker.current {
                    current.success = false;
                }
            }
            _ => {}
        }
    }

    fn change_location(&mut self, location: Option<Location<Touhou7>>) {
        if let Some(location) = location {
            println!("[{0}] Entering {location}", DisplayTime::default());
        }

        let new_info = location.map(|location| ActiveSetInfo {
            start_time: self.now,
            location,
            success: true,
        });
        self.push_attempt(new_info);
    }
}

impl<'a> UpdateTracker<Touhou10> for SetTrackerUpdate<'a, Touhou10> {
    fn push_event(&mut self, event: Event<Touhou10>) {
        if let Some(ref current) = self.tracker.current {
            println!(
                "[{0}] {event:?} at {1}",
                DisplayTime::default(),
                current.location
            );
        } else {
            println!("[{0}] {event:?}", DisplayTime::default());
        }

        match event {
            Event::Miss | Event::Bomb | Event::Continue => {
                if let Some(current) = &mut self.tracker.current {
                    current.success = false;
                }
            }
            _ => {}
        }
    }

    fn change_location(&mut self, location: Option<Location<Touhou10>>) {
        if let Some(location) = location {
            println!("[{0}] Entering {location}", DisplayTime::default());
        }

        let new_info = location.map(|location| ActiveSetInfo {
            start_time: self.now,
            location,
            success: true,
        });
        self.push_attempt(new_info);
    }
}

impl<G> TrackRun<G> for SetTracker<G>
where
    G: TrackableGame,
    Self: TrackGame<G, Output = Vec<(SetKey<G>, Attempt)>>,
{
    fn start_run(
        _time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        _state: <G as TrackableGame>::State,
    ) -> Self {
        println!(
            "[{}] Started run ({} {})",
            DisplayTime::default(),
            shot,
            difficulty
        );

        Self {
            shot,
            difficulty,
            attempts: Vec::new(),
            current: None,
        }
    }

    fn run_cleared(self, time: EventTime, _state: <G as TrackableGame>::State) -> Self::Output {
        println!("[{}] Cleared run", DisplayTime::default());

        self.finish(time, true)
    }

    fn run_exited(self, time: EventTime, _state: <G as TrackableGame>::State) -> Self::Output {
        println!("[{}] Exited run", DisplayTime::default());

        self.finish(time, false)
    }
}

impl<G> TrackStagePractice<G> for SetTracker<G>
where
    G: TrackableGame,
    Self: TrackGame<G, Output = Vec<(SetKey<G>, Attempt)>>,
{
    fn start_stage_practice(
        _time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        stage: touhou::Stage<G>,
        _state: <G as TrackableGame>::State,
    ) -> Self {
        println!(
            "[{}] Started {} practice ({} {})",
            DisplayTime::default(),
            stage,
            shot,
            difficulty
        );

        Self {
            shot,
            difficulty,
            attempts: Vec::new(),
            current: None,
        }
    }

    fn finish_stage_practice(
        self,
        time: EventTime,
        _state: <G as TrackableGame>::State,
    ) -> Self::Output {
        println!("[{}] Finished stage practice", DisplayTime::default());

        self.finish(time, false)
    }
}

impl<G> TrackSpellPractice<G> for SetTracker<G>
where
    G: TrackableGame,
    Self: TrackGame<G, Output = Vec<(SetKey<G>, Attempt)>>,
    SetTracker<G>: TrackGame<G>,
{
    fn start_spell_practice(
        time: EventTime,
        shot: ShotType<G>,
        difficulty: Difficulty<G>,
        location: Location<G>,
        _state: <G as TrackableGame>::State,
    ) -> Self {
        println!(
            "[{}] Started {} practice ({} {})",
            DisplayTime::default(),
            location,
            shot,
            difficulty
        );

        Self {
            shot,
            difficulty,
            attempts: Vec::new(),
            current: Some(ActiveSetInfo {
                start_time: time,
                location,
                success: true,
            }),
        }
    }

    fn finish_spell_practice(
        self,
        time: EventTime,
        _state: <G as TrackableGame>::State,
    ) -> Self::Output {
        println!("[{}] Finished spell practice", DisplayTime::default());

        self.finish(time, true)
    }
}

fn track_loop<G, M, F>(mut new_memory: F) -> Result<(), MemoryReadError<G>>
where
    G: TrackableGame,
    SetTracker<G>: TrackGame<G, Output = Vec<(SetKey<G>, Attempt)>>,
    M: GameMemory<G> + IntoGameTracker<G, SetTracker<G>>,
    F: FnMut() -> Result<Option<M>, MemoryReadError<G>>,
{
    let abbr = G::abbreviation();

    loop {
        println!("[{}] Waiting for {abbr}...", DisplayTime::default());

        let memory = loop {
            if let Some(memory) = new_memory()? {
                println!(
                    "[{0}] Attached to {abbr} process {1}",
                    DisplayTime::default(),
                    memory.pid()
                );
                break memory;
            }

            sleep(Duration::from_millis(500));
        };

        let mut driver = memory.track_games();
        while driver.is_running() {
            if let Some(attempts) = driver.update()? {
                println!("[{0}] Finished {abbr} game:", DisplayTime::default());
                for (key, attempt) in attempts {
                    println!("    {key}: {attempt}");
                }
            }

            sleep(Duration::from_millis(100));
        }

        if let (_, Some(attempts)) = driver.close() {
            println!("[{0}] Finished {abbr} game:", DisplayTime::default());
            for (key, attempt) in attempts {
                println!("    {key}: {attempt}");
            }
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    thread::scope(|s| {
        let th07 = s.spawn(|| track_loop(Th07Memory::new));
        let th10 = s.spawn(|| track_loop(Th10Memory::new));
        th07.join().expect("PCB watcher panicked")?;
        th10.join()
            .expect("MoF watcher panicked")
            .map_err(Into::into)
    })
}
