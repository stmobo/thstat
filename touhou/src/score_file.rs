use std::io::{Error as IOError, ErrorKind, Read};
use std::marker::PhantomData;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::types::{Difficulty, Stage, StageProgress, Touhou17};

pub trait Readable: Sized {
    fn read_from<R: Read>(src: R) -> Result<Self, IOError>;
}

impl<T: Readable, const N: usize> Readable for Box<[T; N]> {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        let mut v = Vec::with_capacity(N);
        for _ in 0..N {
            v.push(T::read_from(&mut src)?);
        }

        match v.try_into() {
            Ok(v) => Ok(v),
            Err(_) => unreachable!("didn't read all elements?"),
        }
    }
}

impl<const N: usize> Readable for [u8; N] {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        let mut ret = [0; N];
        src.read_exact(&mut ret[..])?;
        Ok(ret)
    }
}

impl Readable for u8 {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        src.read_u8()
    }
}

impl Readable for i8 {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        src.read_i8()
    }
}

macro_rules! impl_readable_for_primitives {
    { $(($t:ty, $read_fn:ident)),* } => {
        $(
            impl Readable for $t {
                fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
                    src.$read_fn::<LittleEndian>()
                }
            }
        )*
    };
}

impl_readable_for_primitives! {
    (u16, read_u16),
    (u32, read_u32),
    (u64, read_u64),
    (i16, read_i16),
    (i32, read_i32),
    (i64, read_i64)
}

#[derive(Debug, Clone, Copy)]
pub struct SegmentHeader<const S: usize> {
    signature: [u8; S],
    version: u16,
    checksum: u32,
    size: u32,
}

impl<const S: usize> SegmentHeader<S> {
    pub fn signature(&self) -> &[u8; S] {
        &self.signature
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    pub fn data_size(&self) -> usize {
        (self.size as usize) - S - 10
    }

    pub fn validate_checksum(&self, data: &[u8]) -> bool {
        let sz_bytes = self.size.to_le_bytes();
        sz_bytes
            .iter()
            .map(|x| *x as u32)
            .chain(data.iter().map(|x| *x as u32))
            .reduce(u32::wrapping_add)
            .is_some_and(|computed| computed == self.checksum)
    }
}

impl<const S: usize> Readable for SegmentHeader<S> {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        let mut signature = [0u8; S];
        src.read_exact(&mut signature[..])?;

        Ok(Self {
            signature,
            version: src.read_u16::<LittleEndian>()?,
            checksum: src.read_u32::<LittleEndian>()?,
            size: src.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SpellCardData<T> {
    name: Box<[u8]>,
    play_counts: (u32, u32),             // captures, trials
    practice_counts: Option<(u32, u32)>, // ditto, but not present before TD
    id: u32,
    difficulty: Difficulty,
    practice_score: Option<u32>, // not present before TD
    phantom: PhantomData<T>,
}

impl<T> SpellCardData<T> {
    pub fn raw_name(&self) -> &[u8] {
        &self.name[..]
    }

    pub fn captures(&self) -> u32 {
        self.play_counts.0
    }

    pub fn attempts(&self) -> u32 {
        self.play_counts.1
    }

    pub fn id(&self) -> u32 {
        self.id + 1
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn practice_score(&self) -> Option<u32> {
        self.practice_score
    }

    pub fn practice_captures(&self) -> Option<u32> {
        self.practice_counts.map(|pair| pair.0)
    }

    pub fn practice_attempts(&self) -> Option<u32> {
        self.practice_counts.map(|pair| pair.1)
    }
}

#[derive(Debug, Clone)]
pub struct ScoreData<T> {
    score: u32,
    progress: StageProgress,
    continue_count: u8,
    name: [u8; 10],
    datetime: u32,
    slow_rate: f32,
    phantom: PhantomData<T>,
}

impl<T> ScoreData<T> {
    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn progress(&self) -> StageProgress {
        self.progress
    }

    pub fn continues(&self) -> u8 {
        self.continue_count
    }

    pub fn name(&self) -> &[u8; 10] {
        &self.name
    }

    pub fn datetime(&self) -> u32 {
        self.datetime
    }

    pub fn slow_rate(&self) -> f32 {
        self.slow_rate
    }
}

impl Readable for ScoreData<Touhou17> {
    fn read_from<R: Read>(mut src: R) -> Result<Self, IOError> {
        let score = src.read_u32::<LittleEndian>()?;
        let progress = match src.read_u8()? {
            0 => StageProgress::NotStarted,
            1 => StageProgress::LostAt(Stage::One),
            2 => StageProgress::LostAt(Stage::Two),
            3 => StageProgress::LostAt(Stage::Three),
            4 => StageProgress::LostAt(Stage::Four),
            5 => StageProgress::LostAt(Stage::Five),
            6 => StageProgress::LostAt(Stage::Six),
            7 => StageProgress::LostAt(Stage::Extra),
            8 => StageProgress::AllClear,
            9 => StageProgress::ExtraClear,
            value => {
                return Err(IOError::new(
                    ErrorKind::InvalidData,
                    format!("invalid stage progress value {}", value),
                ));
            }
        };
        let continue_count = src.read_u8()?;
        let mut name = [0; 10];
        src.read_exact(&mut name[..])?;
        let datetime = src.read_u32::<LittleEndian>()?;
        src.read_u32::<LittleEndian>()?;
        let slow_rate = src.read_f32::<LittleEndian>()?;
        src.read_u32::<LittleEndian>()?;

        Ok(Self {
            score,
            progress,
            continue_count,
            name,
            datetime,
            slow_rate,
            phantom: PhantomData,
        })
    }
}
