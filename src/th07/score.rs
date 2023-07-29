use std::convert::TryInto;
use std::fmt::Debug;
use std::io::{self, Cursor, Read};
use std::str;

use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt};

use super::{ShotType as Th07Shot, Touhou7};
use crate::decompress::StreamDecompressor;
use crate::types::{
    Difficulty, ShortDate, ShotType, SpellCard, SpellCardRecord, Stage, StageProgress,
};

macro_rules! impl_getters {
    { $t:ty, $( $field:ident : $field_type:ty ),+ } => {
        impl $t {
            $(
                pub fn $field(&self) -> $field_type {
                    self.$field
                }
            )+
        }
    };
}

macro_rules! access_by_difficulty {
    {$t:ty, $( $field:ident : $field_type:ty ),+} => {
        impl $t {
            $(
                pub fn $field(&self, key: &Difficulty) -> $field_type {
                    match key {
                        Difficulty::Easy => self.$field[0],
                        Difficulty::Normal => self.$field[1],
                        Difficulty::Hard => self.$field[2],
                        Difficulty::Lunatic => self.$field[3],
                        Difficulty::Extra => self.$field[4],
                        Difficulty::Phantasm => self.$field[5]
                    }
                }
            )+
        }
    }
}

macro_rules! access_by_shot {
    {$t:ty, $( $field:ident : $field_type:ty ),+} => {
        impl $t {
            $(
                pub fn $field(&self, key: &Th07Shot) -> $field_type {
                    match *key {
                        Th07Shot::ReimuA => self.$field[0],
                        Th07Shot::ReimuB => self.$field[1],
                        Th07Shot::MarisaA => self.$field[2],
                        Th07Shot::MarisaB => self.$field[3],
                        Th07Shot::SakuyaA => self.$field[4],
                        Th07Shot::SakuyaB => self.$field[5],
                    }
                }
            )+
        }
    }
}

macro_rules! read_array {
    [$x:expr; $n:literal] => {
        {
            let mut tmp = [0; $n];
            for slot in tmp.iter_mut() {
                *slot = $x;
            }
            tmp
        }
    }
}

macro_rules! read_try_into {
    ($t1:ty as $t2:ty : $x:expr) => {{
        use std::io::{self, ErrorKind};
        <$t1 as TryInto<$t2>>::try_into($x).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }};
}

macro_rules! return_none_on_eof {
    ($x:expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoredTime {
    hours: u32,
    minutes: u32,
    seconds: u32,
    milliseconds: u32,
}

impl StoredTime {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        Ok(StoredTime {
            hours: src.read_u32::<LittleEndian>()?,
            minutes: src.read_u32::<LittleEndian>()?,
            seconds: src.read_u32::<LittleEndian>()?,
            milliseconds: src.read_u32::<LittleEndian>()?,
        })
    }
}

impl_getters! {
    StoredTime,
    hours: u32,
    minutes: u32,
    seconds: u32,
    milliseconds: u32
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayCount {
    total_attempts: u32,
    attempts: [u32; 6],
    retries: u32,
    clears: u32,
    continues: u32,
    practices: u32,
}

impl PlayCount {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        Ok(PlayCount {
            total_attempts: src.read_u32::<LittleEndian>()?,
            attempts: read_array![src.read_u32::<LittleEndian>()?; 6],
            retries: src.read_u32::<LittleEndian>()?,
            clears: src.read_u32::<LittleEndian>()?,
            continues: src.read_u32::<LittleEndian>()?,
            practices: src.read_u32::<LittleEndian>()?,
        })
    }
}

access_by_shot! {
    PlayCount,
    attempts: u32
}

impl_getters! {
    PlayCount,
    total_attempts: u32,
    retries: u32,
    clears: u32,
    continues: u32,
    practices: u32
}

#[derive(Debug, Clone)]
pub struct HighScore {
    score: u32,
    slow: f32,
    shot_type: Th07Shot,
    difficulty: Difficulty,
    progress: StageProgress,
    name: [u8; 9],
    date: ShortDate,
    continues: u16,
}

impl HighScore {
    pub fn name(&self) -> Option<&str> {
        str::from_utf8(&self.name[..8]).ok()
    }

    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        src.read_u32::<LittleEndian>()?;

        let score = src.read_u32::<LittleEndian>()?;
        let slow = src.read_f32::<LittleEndian>()?;
        let shot_type = read_try_into!(u8 as Th07Shot : src.read_u8()?)?;
        let difficulty = read_try_into!(u8 as Difficulty : src.read_u8()?)?;

        let progress = match src.read_u8()? {
            0 => StageProgress::NotStarted,
            1 => StageProgress::LostAt(Stage::One),
            2 => StageProgress::LostAt(Stage::Two),
            3 => StageProgress::LostAt(Stage::Three),
            4 => StageProgress::LostAt(Stage::Four),
            5 => StageProgress::LostAt(Stage::Five),
            6 => StageProgress::LostAt(Stage::Six),
            7 => StageProgress::LostAt(Stage::Extra),
            8 => StageProgress::LostAt(Stage::Phantasm),
            99 => StageProgress::AllClear,
            value => bail!("invalid stage progress value {}", value),
        };

        let mut name = [0; 9];
        src.read_exact(&mut name)?;

        let date = ShortDate::read_from(src)?;
        let continues = src.read_u16::<LittleEndian>()?;

        Ok(HighScore {
            score,
            slow,
            shot_type,
            difficulty,
            progress,
            name,
            date,
            continues,
        })
    }
}

impl_getters! {
    HighScore,
    score: u32,
    slow: f32,
    shot_type: Th07Shot,
    difficulty: Difficulty,
    progress: StageProgress,
    date: ShortDate,
    continues: u16
}

#[derive(Debug, Clone, Copy)]
pub struct ClearData {
    story_flags: [u8; 6],
    practice_flags: [u8; 6],
    shot_type: Th07Shot,
}

impl ClearData {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        let mut story_flags = [0; 6];
        let mut practice_flags = [0; 6];

        src.read_u32::<LittleEndian>()?;
        src.read_exact(&mut story_flags)?;
        src.read_exact(&mut practice_flags)?;
        let shot_type = read_try_into!(u8 as Th07Shot : src.read_u32::<LittleEndian>()? as u8)?;

        Ok(ClearData {
            story_flags,
            practice_flags,
            shot_type,
        })
    }
}

impl_getters! {
    ClearData,
    shot_type: Th07Shot
}

access_by_difficulty! { ClearData, story_flags: u8, practice_flags: u8 }

#[derive(Debug, Clone)]
pub struct SpellCardData {
    max_bonuses: [u32; 7],
    card_id: u16,
    card_name: [u8; 0x30],
    attempts: [u16; 7],
    captures: [u16; 7],
}

impl SpellCardData {
    pub fn raw_card_name(&self) -> &[u8] {
        &self.card_name
    }

    pub fn card_name(&self) -> &'static str {
        super::spellcards::resolve_card_name(self.card_id - 1).unwrap()
    }

    pub fn total_max_bonus(&self) -> u32 {
        self.max_bonuses[6]
    }

    pub fn total_attempts(&self) -> u16 {
        self.attempts[6]
    }

    pub fn total_captures(&self) -> u16 {
        self.captures[6]
    }

    pub fn total_capture_rate(&self) -> f64 {
        (self.total_captures() as f64) / (self.total_attempts() as f64)
    }

    pub fn capture_rate(&self, key: &Th07Shot) -> f64 {
        (self.captures(key) as f64) / (self.attempts(key) as f64)
    }

    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        let mut card_name = [0; 0x30];

        src.read_u32::<LittleEndian>()?;

        let max_bonuses = read_array![src.read_u32::<LittleEndian>()?; 7];
        let card_id: u16 = src.read_u16::<LittleEndian>()? + 1;

        src.read_u8()?;
        src.read_exact(&mut card_name)?;
        src.read_u8()?;

        let attempts = read_array![src.read_u16::<LittleEndian>()?; 7];
        let captures = read_array![src.read_u16::<LittleEndian>()?; 7];

        Ok(SpellCardData {
            max_bonuses,
            card_id,
            card_name,
            attempts,
            captures,
        })
    }
}

impl_getters! {
    SpellCardData,
    card_id: u16
}

access_by_shot! {
    SpellCardData,
    max_bonuses: u32,
    attempts: u16,
    captures: u16
}

impl SpellCardRecord<Touhou7> for SpellCardData {
    fn card(&self) -> SpellCard<Touhou7> {
        SpellCard::new(self.card_id).unwrap()
    }

    fn attempts(&self, shot: &ShotType<Touhou7>) -> u32 {
        self.attempts(&shot.as_inner_type()) as u32
    }

    fn captures(&self, shot: &ShotType<Touhou7>) -> u32 {
        self.captures(&shot.as_inner_type()) as u32
    }

    fn max_bonus(&self, shot: &ShotType<Touhou7>) -> u32 {
        self.max_bonuses(&shot.as_inner_type())
    }

    fn total_attempts(&self) -> u32 {
        self.total_attempts() as u32
    }

    fn total_captures(&self) -> u32 {
        self.total_captures() as u32
    }

    fn total_max_bonus(&self) -> u32 {
        self.total_max_bonus()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PracticeData {
    attempts: u32,
    high_score: u32,
    shot_type: Th07Shot,
    difficulty: Difficulty,
    stage: Stage,
}

impl_getters! {
    PracticeData,
    attempts: u32,
    high_score: u32,
    shot_type: Th07Shot,
    difficulty: Difficulty,
    stage: Stage
}

impl PracticeData {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        src.read_u32::<LittleEndian>()?;
        let attempts = src.read_u32::<LittleEndian>()?;
        let high_score = src.read_u32::<LittleEndian>()?;
        let shot_type = read_try_into!(u8 as Th07Shot : src.read_u8()?)?;
        let difficulty = read_try_into!(u8 as Difficulty : src.read_u8()?)?;
        let stage = read_try_into!(u8 as Stage : src.read_u8()?)?;
        src.read_u8()?;

        Ok(PracticeData {
            attempts,
            high_score,
            shot_type,
            difficulty,
            stage,
        })
    }
}

impl crate::types::PracticeRecord<Touhou7> for PracticeData {
    fn stage(&self) -> Stage {
        self.stage
    }

    fn shot_type(&self) -> ShotType<Touhou7> {
        ShotType::from_inner_type(self.shot_type)
    }

    fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    fn high_score(&self) -> u32 {
        self.high_score
    }

    fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[derive(Debug, Clone)]
pub struct PlayData {
    running_time: StoredTime,
    play_time: StoredTime,
    play_counts: Box<[PlayCount; 7]>,
}

impl PlayData {
    pub fn play_counts(&self, key: &Difficulty) -> &PlayCount {
        match key {
            Difficulty::Easy => &self.play_counts[0],
            Difficulty::Normal => &self.play_counts[1],
            Difficulty::Hard => &self.play_counts[2],
            Difficulty::Lunatic => &self.play_counts[3],
            Difficulty::Extra => &self.play_counts[4],
            Difficulty::Phantasm => &self.play_counts[5],
        }
    }

    pub fn total_play_counts(&self) -> &PlayCount {
        &self.play_counts[6]
    }

    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        src.read_u32::<LittleEndian>()?;

        let running_time = StoredTime::read_from(src)?;
        let play_time = StoredTime::read_from(src)?;
        let mut play_counts = Box::new([PlayCount::default(); 7]);

        for slot in play_counts.iter_mut() {
            *slot = PlayCount::read_from(src)?;
        }

        Ok(PlayData {
            running_time,
            play_time,
            play_counts,
        })
    }
}

impl_getters! {
    PlayData,
    running_time: StoredTime,
    play_time: StoredTime
}

#[derive(Debug)]
pub struct Decryptor<R> {
    src: R,
    key: u8,
    checksum: u16,
    target_checksum: u16,
}

impl<R: ReadBytesExt> Decryptor<R> {
    pub fn new(mut src: R) -> Result<Self, anyhow::Error> {
        src.read_u8()?;

        let mut key = src.read_u8()?.rotate_left(3);
        let mut target_checksum = [0; 2];
        src.read_exact(&mut target_checksum)?;

        target_checksum[0] ^= key;
        key = key.wrapping_add(target_checksum[0]).rotate_left(3);

        target_checksum[1] ^= key;
        key = key.wrapping_add(target_checksum[1]).rotate_left(3);

        let target_checksum = u16::from_le_bytes(target_checksum);

        Ok(Self {
            src,
            key,
            checksum: 0,
            target_checksum,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.checksum == self.target_checksum
    }

    pub fn checksum(&self) -> u16 {
        self.checksum
    }

    pub fn target_checksum(&self) -> u16 {
        self.target_checksum
    }
}

impl<R: Read> Read for Decryptor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.src.read(buf)?;

        for x in &mut buf[..n] {
            *x ^= self.key;
            self.key = self.key.wrapping_add(*x).rotate_left(3);
            self.checksum = self.checksum.wrapping_add((*x) as u16);
        }

        Ok(n)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    version: u16,
    header_sz: u32,
    decomp_full_sz: usize,
    decomp_body_sz: usize,
    encoded_body_sz: usize,
}

impl FileHeader {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        let version = src.read_u16::<LittleEndian>()?;
        src.read_u16::<LittleEndian>()?;

        let header_sz = src.read_u32::<LittleEndian>()?;
        src.read_u32::<LittleEndian>()?;

        let decomp_full_sz = src.read_u32::<LittleEndian>()? as usize;
        let decomp_body_sz = src.read_u32::<LittleEndian>()? as usize;
        let encoded_body_sz = src.read_u32::<LittleEndian>()? as usize;

        Ok(Self {
            version,
            header_sz,
            decomp_full_sz,
            decomp_body_sz,
            encoded_body_sz,
        })
    }
}

impl_getters! {
    FileHeader,
    version: u16,
    header_sz: u32,
    decomp_full_sz: usize,
    decomp_body_sz: usize,
    encoded_body_sz: usize
}

#[derive(Clone)]
pub enum Segment {
    Header,
    HighScore(HighScore),
    Clear(ClearData),
    SpellCard(SpellCardData),
    PracticeScore(PracticeData),
    PlayStatus(PlayData),
    LastName([u8; 12]),
    Version([u8; 6]),
    Unknown([u8; 4], usize, usize, Box<[u8]>),
}

impl Segment {
    pub fn signature(&self) -> &[u8; 4] {
        match self {
            Self::Header => b"TH7K",
            Self::HighScore(_) => b"HSCR",
            Self::Clear(_) => b"CLRD",
            Self::SpellCard(_) => b"CATK",
            Self::PracticeScore(_) => b"PSCR",
            Self::PlayStatus(_) => b"PLST",
            Self::LastName(_) => b"LSNM",
            Self::Version(_) => b"VRSM",
            Self::Unknown(sig, _, _, _) => sig,
        }
    }

    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Option<Self>, anyhow::Error> {
        let mut signature = [0; 4];
        return_none_on_eof!(src.read_exact(&mut signature));
        let size1 = return_none_on_eof!(src.read_u16::<LittleEndian>()) as usize;
        let size2 = return_none_on_eof!(src.read_u16::<LittleEndian>()) as usize;

        if size1 <= 8 {
            return Ok(None);
        }

        let mut data = vec![0u8; size1 - 8];
        return_none_on_eof!(src.read_exact(&mut data));

        let mut reader = Cursor::new(data);
        match &signature {
            b"TH7K" => Ok(Self::Header),
            b"HSCR" => HighScore::read_from(&mut reader).map(Self::HighScore),
            b"CLRD" => ClearData::read_from(&mut reader).map(Self::Clear),
            b"CATK" => SpellCardData::read_from(&mut reader).map(Self::SpellCard),
            b"PSCR" => PracticeData::read_from(&mut reader).map(Self::PracticeScore),
            b"PLST" => PlayData::read_from(&mut reader).map(Self::PlayStatus),
            b"LSNM" => {
                let mut name = [0; 12];
                src.read_u32::<LittleEndian>()?;
                src.read_exact(&mut name)?;
                Ok(Self::LastName(name))
            }
            b"VRSM" => {
                let mut version = [0; 6];
                src.read_u16::<LittleEndian>()?;
                src.read_u16::<LittleEndian>()?;
                src.read_exact(&mut version)?;
                src.read_u32::<LittleEndian>()?;
                src.read_u32::<LittleEndian>()?;
                src.read_u16::<LittleEndian>()?;
                Ok(Self::Version(version))
            }
            _ => Ok(Self::Unknown(
                signature,
                size1,
                size2,
                reader.into_inner().into(),
            )),
        }
        .map(Some)
    }
}

impl Debug for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Header => f
                .debug_struct("Segment::Header")
                .field("signature", self.signature())
                .finish(),
            Self::HighScore(d) => f
                .debug_struct("Segment::HighScore")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::Clear(d) => f
                .debug_struct("Segment::Clear")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::SpellCard(d) => f
                .debug_struct("Segment::SpellCard")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::PracticeScore(d) => f
                .debug_struct("Segment::PracticeScore")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::PlayStatus(d) => f
                .debug_struct("Segment::PlayStatus")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::LastName(d) => f
                .debug_struct("Segment::LastName")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::Version(d) => f
                .debug_struct("Segment::Version")
                .field("signature", self.signature())
                .field("data", d)
                .finish(),
            Self::Unknown(sig, s1, s2, d) => f
                .debug_struct("Segment::Unknown")
                .field("signature", sig)
                .field("size_1", s1)
                .field("size_2", s2)
                .field("data", &format!("[{} bytes]", d.len()))
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct ScoreReader<R> {
    header: FileHeader,
    src: StreamDecompressor<Decryptor<R>>,
}

impl<R: Read> ScoreReader<R> {
    pub fn new(src: R) -> Result<Self, anyhow::Error> {
        let mut decryptor = Decryptor::new(src)?;
        let header = FileHeader::read_from(&mut decryptor)?;
        let src = StreamDecompressor::new(decryptor);
        Ok(Self { header, src })
    }

    pub fn header(&self) -> &FileHeader {
        &self.header
    }
}

impl<R: Read> Iterator for ScoreReader<R> {
    type Item = Result<Segment, anyhow::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Segment::read_from(&mut self.src).transpose()
    }
}

#[derive(Debug, Clone)]
pub struct ScoreFile {
    cards: Vec<SpellCardData>,
    practices: Vec<PracticeData>,
}

impl ScoreFile {
    pub fn new<R: Read>(src: R) -> Result<Self, anyhow::Error> {
        let mut cards = Vec::with_capacity(141);
        let mut practices = Vec::new();

        for segment in ScoreReader::new(src)? {
            match segment {
                Ok(Segment::SpellCard(data)) => cards.push(data),
                Ok(Segment::PracticeScore(data)) => practices.push(data),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(Self { cards, practices })
    }
}

impl crate::types::ScoreFile<Touhou7> for ScoreFile {
    fn spell_cards(&self) -> &[SpellCardData] {
        &self.cards[..]
    }

    fn practice_records(&self) -> &[PracticeData] {
        &self.practices[..]
    }
}
