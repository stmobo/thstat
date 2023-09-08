use std::convert::TryInto;
use std::fmt::Debug;
use std::io::{self, Cursor, Read};

use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::crypt::ThCrypt;
use crate::decompress::StreamDecompressor;

const SIGNATURE: &[u8; 4] = b"TH81";

#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    encoded_full_sz: usize,
    encoded_body_sz: usize,
    decoded_body_sz: usize,
}

impl FileHeader {
    pub fn read_from<R: ReadBytesExt>(src: &mut R) -> Result<Self, anyhow::Error> {
        let mut signature: [u8; 4] = [0, 0, 0, 0];
        src.read_exact(&mut signature[..])?;

        if signature != *SIGNATURE {
            bail!("invalid signature")
        }

        let encoded_full_sz = src.read_u32::<LittleEndian>()? as usize;

        src.read_u32::<LittleEndian>()?;
        src.read_u32::<LittleEndian>()?;

        let encoded_body_sz = src.read_u32::<LittleEndian>()? as usize;
        let decoded_body_sz = src.read_u32::<LittleEndian>()? as usize;

        Ok(Self {
            encoded_full_sz,
            encoded_body_sz,
            decoded_body_sz,
        })
    }
}

#[derive(Debug)]
pub struct ScoreReader<R> {
    header: FileHeader,
    src: StreamDecompressor<ThCrypt<R>>,
}

impl<R: Read> ScoreReader<R> {
    pub fn new(mut src: R) -> Result<Self, anyhow::Error> {
        let header = FileHeader::read_from(&mut src)?;
        let decryptor = ThCrypt::new(src, 0xAC, 0x35, 0x10);
        let src = StreamDecompressor::new(decryptor);
        Ok(Self { header, src })
    }

    pub fn header(&self) -> &FileHeader {
        &self.header
    }
}
