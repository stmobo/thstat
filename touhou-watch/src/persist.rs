use std::fs;
use std::fs::File;
use std::io::{BufReader, Error as IOError, ErrorKind, Read, Result as IOResult, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use directories::ProjectDirs;
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use time::OffsetDateTime;
use touhou::Game;

use crate::run::Run;

fn get_project_dirs() -> &'static ProjectDirs {
    static CELL: OnceLock<ProjectDirs> = OnceLock::new();
    CELL.get_or_init(|| {
        ProjectDirs::from("", "FarawayVision", "touhou-watch")
            .expect("could not get project directories")
    })
}

fn get_data_path<G: Game>() -> PathBuf {
    get_project_dirs()
        .data_dir()
        .join(G::GAME_ID.abbreviation())
}

#[derive(Debug)]
pub struct SessionFile<G: Game> {
    path: PathBuf,
    runs: Vec<Run>,
    _phantom: PhantomData<G>,
}

impl<G: Game> SessionFile<G> {
    pub fn new(file_name: impl AsRef<Path>) -> IOResult<Self> {
        let mut path = get_data_path::<G>();
        if !path.is_dir() {
            fs::create_dir_all(&path)?;
        }

        path.push(file_name);
        path.set_extension("json.gz");

        let runs = if path.is_file() {
            let mut reader = File::open(&path).map(BufReader::new).map(GzDecoder::new)?;
            let mut data = Vec::new();
            reader.read_to_end(&mut data)?;
            serde_json::from_slice(&data[..])
                .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?
        } else {
            Vec::new()
        };

        Ok(Self {
            path,
            runs,
            _phantom: PhantomData,
        })
    }

    pub fn new_default() -> IOResult<Self> {
        let now = OffsetDateTime::now_utc();
        let month: u8 = now.month().into();

        Self::new(format!(
            "{:04}-{:02}-{:02}T{:02}_{:02}_{:02}Z",
            now.year(),
            month,
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        ))
    }

    pub fn save(&self) -> IOResult<()> {
        let mut file = GzEncoder::new(File::create(&self.path)?, Compression::default());
        let data =
            serde_json::to_vec(&self.runs).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        file.write_all(&data[..])
    }

    pub fn add_run(&mut self, run: Run) {
        let idx = self
            .runs
            .binary_search_by_key(&run.start_time(), |r| r.start_time())
            .unwrap_or_else(std::convert::identity);
        self.runs.insert(idx, run);
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }
}
