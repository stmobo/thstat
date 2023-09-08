use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{self, Cursor, ErrorKind, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use super::{Difficulty, ShotType, SpellId, Stage, Touhou8};
use crate::crypt::ThCrypt;
use crate::decompress::StreamDecompressor;
use crate::th07::score::FileHeader;
use crate::types::{
    Difficulty as DifficultyWrapper, PracticeRecord, ShortDate, ShotType as ShotWrapper, SpellCard,
    SpellCardRecord, SpellPracticeRecord, Stage as StageWrapper, StageProgress,
};

fn read_raw_buffer<const N: usize, R: Read>(mut src: R) -> io::Result<Box<[u8]>> {
    let mut buf = vec![0u8; N];
    src.read_exact(&mut buf[..])?;
    Ok(buf.into())
}

fn skip_bytes<const N: usize, R: Read>(mut src: R) -> io::Result<()> {
    let mut buf = [0u8; N];
    src.read_exact(&mut buf[..])
}

macro_rules! read_then_skip_bytes {
    ($src:expr, $read:expr, $skip:literal) => {{
        let r = $read;
        skip_bytes::<$skip, _>($src)?;
        r
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

fn try_into_or_io_error<T, U>(kind: ErrorKind) -> impl FnOnce(T) -> io::Result<U>
where
    T: TryInto<U>,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    move |val| {
        val.try_into()
            .map_err(move |error| io::Error::new(kind, error))
    }
}

#[derive(Debug)]
enum DecryptorState<R> {
    ThcryptActive(ThCrypt<R>),
    ReadRest(R),
    Working,
}

#[derive(Debug)]
pub struct Decryptor<R> {
    crypt: DecryptorState<R>,
    acc: u8,
    checksum: u16,
    target_checksum: u16,
}

impl<R: Read> Decryptor<R> {
    fn new(src: R) -> io::Result<Self> {
        let mut crypt = ThCrypt::new(src, 0x59, 0x79, 0x0100, Some(0x0C00));
        let mut init_bytes = [0u8; 4];

        crypt.read_exact(&mut init_bytes[..])?;

        let mut acc = init_bytes[1].rotate_left(3);

        init_bytes[2] ^= acc;
        acc = acc.wrapping_add(init_bytes[2]).rotate_left(3);

        init_bytes[3] ^= acc;
        acc = acc.wrapping_add(init_bytes[3]).rotate_left(3);

        Ok(Self {
            crypt: DecryptorState::ThcryptActive(crypt),
            acc,
            target_checksum: u16::from_le_bytes([init_bytes[2], init_bytes[3]]),
            checksum: 0,
        })
    }

    pub fn valid_checksum(&self) -> bool {
        self.checksum == self.target_checksum
    }
}

impl<R: Read> Read for Decryptor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = match std::mem::replace(&mut self.crypt, DecryptorState::Working) {
            DecryptorState::ThcryptActive(mut crypt) => {
                let n = crypt.read(buf)?;
                if crypt.at_limit() {
                    self.crypt = DecryptorState::ReadRest(crypt.unwrap());
                } else {
                    self.crypt = DecryptorState::ThcryptActive(crypt);
                }
                n
            }
            DecryptorState::ReadRest(mut src) => {
                let n = src.read(buf)?;
                self.crypt = DecryptorState::ReadRest(src);
                n
            }
            DecryptorState::Working => unreachable!(),
        };

        for x in &mut buf[..n] {
            *x ^= self.acc;
            self.acc = self.acc.wrapping_add(*x).rotate_left(3);

            self.checksum = self.checksum.wrapping_add((*x) as u16);
        }

        Ok(n)
    }
}

#[derive(Debug, Clone)]
pub struct HighScore {
    score: u32,
    slow: f32,
    shot_type: ShotType,
    difficulty: Difficulty,
    progress: StageProgress<Touhou8>,
    name: [u8; 9],
    date: ShortDate,
    continues: u16,
    player_num: u8,
    play_time: u32,
    point_item: u32,
    miss_count: u32,
    bomb_count: u32,
    last_spells: u32,
    pause_count: u32,
    time_points: u32,
    human_rate: u32,
    card_flags: Box<[u8]>, // 222 bytes
}

impl HighScore {
    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn slow(&self) -> f32 {
        self.slow
    }

    pub fn shot_type(&self) -> ShotType {
        self.shot_type
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn progress(&self) -> StageProgress<Touhou8> {
        self.progress
    }

    pub fn name(&self) -> &[u8] {
        &self.name[..]
    }

    pub fn date(&self) -> ShortDate {
        self.date
    }

    pub fn continues(&self) -> u16 {
        self.continues
    }

    pub fn player_num(&self) -> u8 {
        self.player_num
    }

    pub fn play_time(&self) -> u32 {
        self.play_time
    }

    pub fn point_item(&self) -> u32 {
        self.point_item
    }

    pub fn miss_count(&self) -> u32 {
        self.miss_count
    }

    pub fn bomb_count(&self) -> u32 {
        self.bomb_count
    }

    pub fn last_spells(&self) -> u32 {
        self.last_spells
    }

    pub fn pause_count(&self) -> u32 {
        self.pause_count
    }

    pub fn time_points(&self) -> u32 {
        self.time_points
    }

    pub fn human_rate(&self) -> u32 {
        self.human_rate
    }

    pub fn card_flags(&self) -> &[u8] {
        &self.card_flags[..]
    }

    pub fn read_from<R: Read>(mut src: R) -> io::Result<Self> {
        skip_bytes::<4, &mut R>(&mut src)?;

        Ok(Self {
            score: src.read_u32::<LittleEndian>()?,
            slow: src.read_f32::<LittleEndian>()?,
            shot_type: src
                .read_u8()
                .and_then(try_into_or_io_error(ErrorKind::InvalidData))?,
            difficulty: src
                .read_u8()
                .and_then(try_into_or_io_error(ErrorKind::InvalidData))?,
            progress: match src.read_u8()? {
                0 => StageProgress::LostAt(StageWrapper::new(Stage::One)),
                1 => StageProgress::LostAt(StageWrapper::new(Stage::Two)),
                2 => StageProgress::LostAt(StageWrapper::new(Stage::Three)),
                3 => StageProgress::LostAt(StageWrapper::new(Stage::FourA)),
                4 => StageProgress::LostAt(StageWrapper::new(Stage::FourB)),
                5 => StageProgress::LostAt(StageWrapper::new(Stage::Five)),
                6 => StageProgress::LostAt(StageWrapper::new(Stage::FinalA)),
                7 => StageProgress::LostAt(StageWrapper::new(Stage::FinalB)),
                8 => StageProgress::LostAt(StageWrapper::new(Stage::Extra)),
                99 => StageProgress::AllClear,
                value => {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid stage progress value {}", value),
                    ));
                }
            },
            name: {
                let mut buf = [0u8; 9];
                src.read_exact(&mut buf[..])?;
                buf
            },
            date: ShortDate::read_from(&mut src)?,
            continues: read_then_skip_bytes!(&mut src, src.read_u16::<LittleEndian>()?, 0x1C),
            player_num: read_then_skip_bytes!(&mut src, src.read_u8()?, 0x1F),
            play_time: src.read_u32::<LittleEndian>()?,
            point_item: read_then_skip_bytes!(&mut src, src.read_u32::<LittleEndian>()?, 4),
            miss_count: src.read_u32::<LittleEndian>()?,
            bomb_count: src.read_u32::<LittleEndian>()?,
            last_spells: src.read_u32::<LittleEndian>()?,
            pause_count: src.read_u32::<LittleEndian>()?,
            time_points: src.read_u32::<LittleEndian>()?,
            human_rate: src.read_u32::<LittleEndian>()?,
            card_flags: read_then_skip_bytes!(
                &mut src,
                {
                    let mut buf = vec![0u8; 222];
                    src.read_exact(&mut buf[..])?;
                    buf.into()
                },
                2
            ),
        })
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SpellCardCareer {
    max_bonus: (u32, u32), // pairs of story/practice values
    attempts: (u32, u32),
    captures: (u32, u32),
}

impl SpellCardCareer {
    pub fn max_bonus(&self, practice: bool) -> u32 {
        if practice {
            self.max_bonus.1
        } else {
            self.max_bonus.0
        }
    }

    pub fn attempts(&self, practice: bool) -> u32 {
        if practice {
            self.attempts.1
        } else {
            self.attempts.0
        }
    }

    pub fn captures(&self, practice: bool) -> u32 {
        if practice {
            self.captures.1
        } else {
            self.captures.0
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpellCardData {
    card_id: SpellId,
    difficulty: Difficulty,
    card_name: Box<[u8]>,  // 0x30 bytes, CP932
    enemy_name: Box<[u8]>, //  0x30 bytes, probably also CP932?
    comment: Box<[u8]>,    // 0x80 bytes
    career_stats: Vec<SpellCardCareer>,
    total_stats: SpellCardCareer,
}

impl SpellCardData {
    pub fn card_id(&self) -> SpellId {
        self.card_id
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn card_name(&self) -> &[u8] {
        &self.card_name[..]
    }

    pub fn enemy_name(&self) -> &[u8] {
        &self.enemy_name[..]
    }

    pub fn comment(&self) -> &[u8] {
        &self.comment[..]
    }

    pub fn shot_stats(&self, shot: &ShotType) -> &SpellCardCareer {
        let idx: usize = shot.into();
        &self.career_stats[idx]
    }

    pub fn iter_shot_stats(&self) -> impl Iterator<Item = (ShotType, &SpellCardCareer)> + '_ {
        ShotType::iter().zip(self.career_stats.iter())
    }

    pub fn total_stats(&self) -> &SpellCardCareer {
        &self.total_stats
    }

    pub fn read_from<R: Read>(mut src: R) -> io::Result<Self> {
        src.read_u32::<LittleEndian>()?;

        let card_id = src
            .read_u16::<LittleEndian>()
            .map(|x| (x as u32) + 1)
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        src.read_u8()?;

        let difficulty = src
            .read_u8()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        let card_name = read_raw_buffer::<0x30, _>(&mut src)?;
        let enemy_name = read_raw_buffer::<0x30, _>(&mut src)?;
        let comment = read_raw_buffer::<0x80, _>(&mut src)?;

        let mut arrays = [[0u32; 13]; 6];
        for subarray in arrays.iter_mut() {
            for elem in subarray.iter_mut() {
                *elem = src.read_u32::<LittleEndian>()?;
            }
        }

        src.read_u32::<LittleEndian>()?;

        let mut career_stats = Vec::with_capacity(12);
        for i in 0..12 {
            career_stats.push(SpellCardCareer {
                max_bonus: (arrays[0][i], arrays[3][i]),
                attempts: (arrays[1][i], arrays[4][i]),
                captures: (arrays[2][i], arrays[5][i]),
            })
        }

        let total_stats = SpellCardCareer {
            max_bonus: (arrays[0][12], arrays[3][12]),
            attempts: (arrays[1][12], arrays[4][12]),
            captures: (arrays[2][12], arrays[5][12]),
        };

        Ok(Self {
            card_id,
            difficulty,
            card_name,
            enemy_name,
            comment,
            career_stats,
            total_stats,
        })
    }
}

impl SpellCardRecord<Touhou8> for SpellCardData {
    fn card(&self) -> SpellCard<Touhou8> {
        SpellCard::new(self.card_id)
    }

    fn shot_types(&self) -> &[ShotWrapper<Touhou8>] {
        &Touhou8::SHOT_TYPES[..]
    }

    fn attempts(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].attempts(false)
    }

    fn captures(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].captures(false)
    }

    fn max_bonus(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].max_bonus(false)
    }

    fn total_attempts(&self) -> u32 {
        self.total_stats.attempts(false)
    }

    fn total_captures(&self) -> u32 {
        self.total_stats.captures(false)
    }

    fn total_max_bonus(&self) -> u32 {
        self.total_stats.max_bonus(false)
    }
}

impl SpellPracticeRecord<Touhou8> for SpellCardData {
    fn practice_attempts(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].attempts(true)
    }

    fn practice_captures(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].captures(true)
    }

    fn practice_max_bonus(&self, shot: &ShotWrapper<Touhou8>) -> u32 {
        let idx: usize = shot.unwrap().into();
        self.career_stats[idx].max_bonus(true)
    }

    fn practice_total_attempts(&self) -> u32 {
        self.total_stats.attempts(true)
    }

    fn practice_total_captures(&self) -> u32 {
        self.total_stats.captures(true)
    }

    fn practice_total_max_bonus(&self) -> u32 {
        self.total_stats.max_bonus(true)
    }
}

#[derive(Debug, Clone)]
pub struct PracticeScore {
    shot_type: ShotType,
    stage: Stage,
    difficulty: Difficulty,
    high_score: u32,
    attempts: u32,
}

#[derive(Debug, Clone)]
pub struct PracticeData {
    practice_data: HashMap<(Stage, Difficulty), PracticeScore>,
    shot_type: ShotType,
}

impl PracticeData {
    pub fn shot_type(&self) -> ShotType {
        self.shot_type
    }

    pub fn get_practice_data(
        &self,
        stage: Stage,
        difficulty: Difficulty,
    ) -> Option<&PracticeScore> {
        self.practice_data.get(&(stage, difficulty))
    }

    pub fn iter_practice_data(&self) -> impl Iterator<Item = &PracticeScore> + '_ {
        self.practice_data.values()
    }

    pub fn read_from<R: Read>(mut src: R) -> io::Result<Self> {
        static READ_STAGES: [Stage; 9] = [
            Stage::One,
            Stage::Two,
            Stage::Three,
            Stage::FourA,
            Stage::FourB,
            Stage::Five,
            Stage::FinalA,
            Stage::FinalB,
            Stage::Extra,
        ];

        static READ_DIFFICULTIES: [Difficulty; 5] = [
            Difficulty::Easy,
            Difficulty::Normal,
            Difficulty::Hard,
            Difficulty::Lunatic,
            Difficulty::Extra,
        ];

        skip_bytes::<4, _>(&mut src)?;

        let mut play_counts = [0u32; 45];
        let mut high_scores = [0u32; 45];

        for arr in [&mut play_counts, &mut high_scores] {
            for elem in arr.iter_mut() {
                *elem = src.read_u32::<LittleEndian>()?;
            }
        }

        let shot_type = src
            .read_u8()
            .and_then(try_into_or_io_error(ErrorKind::InvalidData))?;

        skip_bytes::<3, _>(&mut src)?;

        let keys = READ_STAGES.into_iter().flat_map(|stage| {
            READ_DIFFICULTIES
                .into_iter()
                .map(move |difficulty| (stage, difficulty))
        });

        let mut practice_data = HashMap::new();
        for (key, pair) in keys.zip(play_counts.into_iter().zip(high_scores)) {
            practice_data.insert(
                key,
                PracticeScore {
                    shot_type,
                    stage: key.0,
                    difficulty: key.1,
                    high_score: pair.1,
                    attempts: pair.0,
                },
            );
        }

        Ok(Self {
            practice_data,
            shot_type,
        })
    }
}

impl PracticeRecord<Touhou8> for PracticeScore {
    fn high_score(&self) -> u32 {
        self.high_score
    }

    fn attempts(&self) -> u32 {
        self.attempts
    }

    fn shot_type(&self) -> ShotWrapper<Touhou8> {
        ShotWrapper::new(self.shot_type)
    }

    fn difficulty(&self) -> DifficultyWrapper<Touhou8> {
        DifficultyWrapper::new(self.difficulty)
    }

    fn stage(&self) -> StageWrapper<Touhou8> {
        StageWrapper::new(self.stage)
    }
}

#[derive(Clone)]
pub enum Segment {
    Header,
    HighScore(HighScore),
    SpellCard(SpellCardData),
    Practice(PracticeData),
    Unknown([u8; 4], usize, usize, Box<[u8]>),
}

impl Segment {
    pub fn signature(&self) -> &[u8; 4] {
        match self {
            Self::Header => b"TH8K",
            Self::HighScore(_) => b"HSCR",
            Self::SpellCard(_) => b"CATK",
            Self::Practice(_) => b"PSCR",
            Self::Unknown(sig, _, _, _) => sig,
        }
    }

    fn signature_string(&self) -> String {
        let v: Vec<u8> = self
            .signature()
            .iter()
            .copied()
            .flat_map(std::ascii::escape_default)
            .collect();
        String::from_utf8(v).unwrap()
    }

    pub fn read_from<R: Read>(mut src: R) -> io::Result<Option<Self>> {
        let mut signature = [0; 4];
        return_none_on_eof!(src.read_exact(&mut signature));
        let size1 = return_none_on_eof!(src.read_u16::<LittleEndian>()) as usize;
        let size2 = return_none_on_eof!(src.read_u16::<LittleEndian>()) as usize;

        if size1 <= 8 {
            return Self::read_from(src);
        }

        let mut data = vec![0u8; size1 - 8];
        return_none_on_eof!(src.read_exact(&mut data));

        let mut reader = Cursor::new(data);
        match &signature {
            b"TH8K" => Ok(Self::Header),
            b"HSCR" => HighScore::read_from(&mut reader).map(Self::HighScore),
            b"CATK" => SpellCardData::read_from(&mut reader).map(Self::SpellCard),
            b"PSCR" => PracticeData::read_from(&mut reader).map(Self::Practice),
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
                .field("signature", &self.signature_string())
                .finish(),
            Self::HighScore(d) => f
                .debug_struct("Segment::HighScore")
                .field("signature", &self.signature_string())
                .field("data", d)
                .finish(),
            Self::SpellCard(d) => f
                .debug_struct("Segment::SpellCard")
                .field("signature", &self.signature_string())
                .field("data", d)
                .finish(),
            Self::Practice(d) => f
                .debug_struct("Segment::Practice")
                .field("signature", &self.signature_string())
                .field("data", d)
                .finish(),
            Self::Unknown(_, s1, s2, d) => f
                .debug_struct("Segment::Unknown")
                .field("signature", &self.signature_string())
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
    pub fn new(src: R) -> Result<Self, io::Error> {
        let mut decryptor = Decryptor::new(src)?;
        let header = FileHeader::read_from(&mut decryptor)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        let src = StreamDecompressor::new(decryptor);
        Ok(Self { header, src })
    }

    pub fn header(&self) -> &FileHeader {
        &self.header
    }
}

impl<R: Read> Iterator for ScoreReader<R> {
    type Item = Result<Segment, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Segment::read_from(&mut self.src).transpose()
    }
}

#[derive(Debug, Clone)]
pub struct ScoreFile {
    cards: Vec<SpellCardData>,
    practices: Vec<PracticeScore>,
}

impl ScoreFile {
    pub fn new<R: Read>(src: R) -> Result<Self, io::Error> {
        let mut cards = Vec::with_capacity(141);
        let mut practices = Vec::new();

        for segment in ScoreReader::new(src)? {
            match segment {
                Ok(Segment::SpellCard(data)) => cards.push(data),
                Ok(Segment::Practice(data)) => practices.extend(data.practice_data.into_values()),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(Self { cards, practices })
    }
}

impl crate::types::ScoreFile<Touhou8> for ScoreFile {
    fn spell_cards(&self) -> &[SpellCardData] {
        &self.cards[..]
    }

    fn practice_records(&self) -> &[PracticeScore] {
        &self.practices[..]
    }
}
