use byteorder::{LittleEndian, ReadBytesExt};


use std::io::{self, Cursor, ErrorKind, Read, Seek, SeekFrom};
use std::ops::{BitAnd, BitOr, Not};

use crate::decompress::Decompressor;
use crate::types::{Character, Difficulty};

use super::ShotType;

macro_rules! test_bit {
    ([$t:ident, $tmp:ident, $src:expr], $(($name:ident, $mask:expr)),+) => {
        impl $t {
            $(
                pub fn $name(&self) -> bool {
                    let $tmp = self;
                    ($src & $mask) != 0
                }
            )+
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Touhou7Input {
    directions: u8,
    unk1: u8,
    state: u8,
    unk3: u8,
}

test_bit!(
    [Touhou7Input, x, x.directions],
    (fire, 0x01),
    (focus, 0x02),
    (up, 0x10),
    (down, 0x20),
    (left, 0x40),
    (right, 0x80)
);

test_bit!(
    [Touhou7Input, x, x.state],
    (bomb, 0x01),
    (miss, 0x02),
    (dead, 0x04),
    (border, 0x08)
);

impl Touhou7Input {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> io::Result<Self> {
        Ok(Self {
            directions: src.read_u8()?,
            unk1: src.read_u8()?,
            state: src.read_u8()?,
            unk3: src.read_u8()?,
        })
    }

    pub fn from_slice(src: &[u8]) -> impl Iterator<Item = Touhou7Input> + '_ {
        src.chunks_exact(4).map(move |chunk| Touhou7Input {
            directions: chunk[0],
            unk1: chunk[1],
            state: chunk[2],
            unk3: chunk[3],
        })
    }

    pub fn from_slice_debounced(
        src: &[u8],
        debounce: u8,
    ) -> impl Iterator<Item = Touhou7Input> + '_ {
        let mut tm = [[0u8; 8]; 4];

        src.chunks_exact(4).map(move |chunk| {
            let mut out_data = [0; 4];

            for (b, (timers, out)) in chunk
                .iter()
                .copied()
                .zip(tm.iter_mut().zip(out_data.iter_mut()))
            {
                let mut out_b = 0;

                for (i, slot) in timers.iter_mut().enumerate() {
                    let mask = 1 << (i as u8);
                    if (b & mask) != 0 {
                        *slot = debounce;
                    }

                    if *slot > 0 {
                        out_b |= mask;
                        *slot -= 1;
                    }
                }

                *out = out_b;
            }

            Touhou7Input {
                directions: out_data[0],
                unk1: out_data[1],
                state: out_data[2],
                unk3: out_data[3],
            }
        })
    }
}

impl BitOr for Touhou7Input {
    type Output = Touhou7Input;

    fn bitor(self, rhs: Self) -> Self::Output {
        Touhou7Input {
            directions: self.directions | rhs.directions,
            unk1: self.unk1 | rhs.unk1,
            state: self.state | rhs.state,
            unk3: self.unk3 | rhs.unk3,
        }
    }
}

impl BitAnd for Touhou7Input {
    type Output = Touhou7Input;

    fn bitand(self, rhs: Self) -> Self::Output {
        Touhou7Input {
            directions: self.directions & rhs.directions,
            unk1: self.unk1 & rhs.unk1,
            state: self.state & rhs.state,
            unk3: self.unk3 & rhs.unk3,
        }
    }
}

impl Not for Touhou7Input {
    type Output = Touhou7Input;

    fn not(self) -> Self::Output {
        Touhou7Input {
            directions: !self.directions,
            unk1: !self.unk1,
            state: !self.state,
            unk3: !self.unk3,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BitCounter {
    frames: [u64; 8],
    rising_edges: [u64; 8],
    falling_edges: [u64; 8],
    last_state: u8,
}

impl BitCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_frame(&mut self, inputs: u8) {
        let rising = inputs & !self.last_state;
        let falling = self.last_state & !inputs;

        for i in 0..8 {
            let mask = 1 << (i as u8);

            if (inputs & mask) != 0 {
                self.frames[i] += 1;
            }

            if (rising & mask) != 0 {
                self.rising_edges[i] += 1;
            }

            if (falling & mask) != 0 {
                self.falling_edges[i] += 1;
            }
        }

        self.last_state = inputs;
    }

    pub fn is_empty(&self) -> bool {
        !self.frames.iter().any(|x| *x > 0)
    }

    fn print_counter(ctr: &[u64], unit: &str) {
        let idxs = [0, 1, 2, 3, 4, 5, 6, 7];
        // idxs.sort_by(|a, b| ctr[*a].cmp(&ctr[*b]).reverse());

        for idx in idxs {
            if ctr[idx] > 0 {
                println!("{} | {} {}", idx, ctr[idx], unit);
            }
        }
    }

    pub fn print_frames(&self, title: &str) {
        if !self.is_empty() {
            println!("{} Key Frames", title);
            println!("------------------------------");
            Self::print_counter(&self.frames, "frames");
        }
    }

    pub fn print_rising(&self, title: &str) {
        if !self.is_empty() {
            println!("{} Key Rising Edges", title);
            println!("------------------------------");
            Self::print_counter(&self.rising_edges, "edges");
        }
    }

    pub fn print_falling(&self, title: &str) {
        if !self.is_empty() {
            println!("{} Key Falling Edges", title);
            println!("------------------------------");
            Self::print_counter(&self.falling_edges, "edges");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Touhou7ReplayStage {
    pub stage: u8,
    pub end_score: u32,
    pub point_item_val: u32,
    pub point_item_count: u32,
    pub cherrymax: u32,
    pub graze: u32,
    pub power: u8,
    pub start_bombs: u8,
    pub used_bombs: u8,
    pub start_lives: u8,
    pub deaths: u8,
    pub borders: u8,
    pub avg_fps: f64,
}

impl Touhou7ReplayStage {
    pub fn read_from<R: ReadBytesExt + Seek>(
        stage: u8,
        _character: Character,
        avg_fps: f64,
        src: &mut R,
        inputs: &[u8],
    ) -> io::Result<Self> {
        let end_score = src.read_u32::<LittleEndian>()?.saturating_mul(10);
        let point_item_count = src.read_u32::<LittleEndian>()?;
        let point_item_val = src.read_u32::<LittleEndian>()?;
        let cherrymax = src.read_u32::<LittleEndian>()?;
        src.seek(SeekFrom::Current(4))?;
        let graze = src.read_u32::<LittleEndian>()?;
        src.seek(SeekFrom::Current(0x0A))?;
        let power = src.read_u8()?;
        let start_lives = src.read_u8()?;
        let start_bombs = src.read_u8()?;

        let mut deaths = 0;
        let mut used_bombs = 0;
        let mut borders = 0;

        let mut input_iter = Touhou7Input::from_slice(inputs).skip(15);
        let mut prev_input = input_iter.next().unwrap();

        for (_frame, input) in input_iter.enumerate() {
            let rising = input & !prev_input;

            if rising.dead() {
                deaths += 1;
            }

            if rising.bomb() {
                used_bombs += 1;
            }

            if rising.border() {
                borders += 1;
            }

            prev_input = input;
        }

        Ok(Self {
            stage,
            end_score,
            point_item_val,
            point_item_count,
            cherrymax,
            graze,
            power,
            start_lives,
            deaths,
            start_bombs,
            used_bombs,
            borders,
            avg_fps,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Touhou7Replay {
    pub character: ShotType,
    pub difficulty: Difficulty,
    pub date: String,
    pub name: String,
    pub score: u32,
    pub stages: Vec<Touhou7ReplayStage>,
}

impl Touhou7Replay {
    fn read_file<R: Read>(mut src: R) -> io::Result<(Cursor<Box<[u8]>>, Cursor<Box<[u8]>>)> {
        let mut header: Box<[u8]> = vec![0u8; 0x54].into();
        src.read_exact(&mut header[..])?;

        let mut raw_data = Vec::new();
        let file_len = io::copy(&mut src, &mut raw_data)?;

        let mut key = header[0x0D];
        for x in header.iter_mut().skip(0x10).chain(raw_data.iter_mut()) {
            *x = x.wrapping_sub(key);
            key = key.wrapping_add(7);
        }

        let raw_len = u32::from_le_bytes(header[0x14..0x18].try_into().unwrap()) as u64;
        let dec_len = u32::from_le_bytes(header[0x18..0x1C].try_into().unwrap()) as usize;
        if file_len < raw_len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "file is truncated (read {} bytes of user data, expected {})",
                    file_len, raw_len
                ),
            ));
        }

        let dec_data = {
            let mut dec = Decompressor::new(&raw_data[..(raw_len as usize)]);
            let mut buf = vec![0u8; dec_len];
            dec.read_exact(&mut buf[..])?;
            buf.into()
        };

        Ok((Cursor::new(header), Cursor::new(dec_data)))
    }

    pub fn decode<R: Read>(src: R) -> io::Result<Self> {
        let mut header;
        let mut user_data;
        (header, user_data) = Self::read_file(src)?;

        let mut score_offset = [None; 7];
        for (i, slot) in score_offset.iter_mut().enumerate() {
            header.seek(SeekFrom::Start(0x1C + (i * 4) as u64))?;
            let score = header.read_u32::<LittleEndian>()?;

            header.seek(SeekFrom::Start(0x38 + (i * 4) as u64))?;
            let fps = header.read_u32::<LittleEndian>()?;

            if (score >= 0x54) && (fps >= 0x54) {
                *slot = Some((i as u8, (score - 0x54) as u64, (fps - 0x54) as u64));
            }
        }

        user_data.seek(SeekFrom::Start(2))?;
        let character = ShotType::try_from(user_data.read_u8()?)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
        let difficulty = Difficulty::try_from(user_data.read_u8()?)
            .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

        user_data.seek(SeekFrom::Start(0x18))?;
        let score = user_data.read_u32::<LittleEndian>()?;

        let (date, name) = {
            let data_slice = user_data.get_ref();
            let date = std::str::from_utf8(&data_slice[0x04..0x09])
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            let name = std::str::from_utf8(&data_slice[0x0A..0x12])
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;

            (date.to_string(), name.to_string())
        };

        let offsets: Vec<_> = score_offset.into_iter().flatten().collect();
        let mut stages = Vec::new();

        for (idx, (stage, score_offset, fps_start)) in offsets.iter().cloned().enumerate() {
            let input_start = (score_offset + 0x28) as usize;
            let input_end = if idx == (offsets.len() - 1) {
                offsets[0].2
            } else {
                offsets[idx + 1].1
            } as usize;

            let fps_data = if idx == (offsets.len() - 1) {
                &user_data.get_ref()[(fps_start as usize)..]
            } else {
                &user_data.get_ref()[(fps_start as usize)..(offsets[idx + 1].2 as usize)]
            };

            let mut stage_cursor = Cursor::new(&user_data.get_ref()[..]);
            stage_cursor.seek(SeekFrom::Start(score_offset))?;

            let input_data = &user_data.get_ref()[input_start..input_end];

            let fps_sum = fps_data.iter().copied().map(|x| x as u32).sum::<u32>() as f64;
            let avg_fps = fps_sum / (fps_data.len() as f64);

            stages.push(Touhou7ReplayStage::read_from(
                stage + 1,
                character.character,
                avg_fps,
                &mut stage_cursor,
                input_data,
            )?);
        }

        Ok(Self {
            character,
            difficulty,
            date,
            name,
            score,
            stages,
        })
    }
}
