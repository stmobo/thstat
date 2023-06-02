use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, ErrorKind, Read};

#[derive(Debug, Clone)]
enum DecompressorState {
    Init,
    OutputSingle(u8),
    OutputMultiple([u8; 19], usize, usize),
}

#[derive(Debug, Clone)]
pub struct Decompressor<'a> {
    src: &'a [u8],
    cur_byte: Option<u8>,
    cur_bit: u8,
    output_pos: usize,
    dict: Box<[u8]>,
    state: Option<DecompressorState>,
}

impl<'a> Decompressor<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let dict = vec![0u8; 0x2010].into();
        Self {
            src,
            cur_byte: None,
            cur_bit: 0x80,
            output_pos: 0,
            dict,
            state: Some(DecompressorState::Init),
        }
    }

    pub fn ensure_next_byte(&mut self) -> Option<u8> {
        if self.cur_byte.is_none() {
            if let Some((next, rest)) = self.src.split_first() {
                self.cur_byte = Some(*next);
                self.src = rest;
            }
        }

        self.cur_byte
    }

    fn next_bit(&mut self) -> Option<bool> {
        let ret = (self.ensure_next_byte()? & self.cur_bit) != 0;
        if self.cur_bit == 1 {
            self.cur_bit = 0x80;
            self.cur_byte = None;
        } else {
            self.cur_bit >>= 1;
        }

        Some(ret)
    }

    fn next_bits<const N: usize>(&mut self) -> Option<u16> {
        debug_assert!(N <= 16);

        let mut result = 0;
        for _ in 0..N {
            result = (result << 1) | (self.next_bit()? as u16);
        }

        Some(result)
    }

    fn decode_next(&mut self) -> Option<DecompressorState> {
        if self.next_bit()? {
            let val = self.next_bits::<8>()? as u8;

            self.dict[self.output_pos & 0x1FFF] = val;
            self.output_pos += 1;

            Some(DecompressorState::OutputSingle(val))
        } else {
            let idx = self.next_bits::<13>()?.wrapping_sub(1) as usize;
            let n = (self.next_bits::<4>()? + 3) as usize;

            assert!(n <= 19, "invalid decode length");

            let mut decode_buf = [0; 19];
            for (i, out) in decode_buf.iter_mut().take(n).enumerate() {
                self.dict[self.output_pos & 0x1FFF] = self.dict[idx + i];
                *out = self.dict[idx + i];
                self.output_pos += 1;
            }

            Some(DecompressorState::OutputMultiple(decode_buf, 0, n))
        }
    }
}

impl<'a> Read for Decompressor<'a> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        while let Some(state) = self.state.take() {
            if buf.is_empty() {
                self.state = Some(state);
                break;
            }

            self.state = match state {
                DecompressorState::Init => self.decode_next(),
                DecompressorState::OutputSingle(val) => {
                    if let Some((next_out, rest)) = buf.split_first_mut() {
                        *next_out = val;
                        buf = rest;
                        n += 1;
                        self.decode_next()
                    } else {
                        self.state = Some(DecompressorState::OutputSingle(val));
                        break;
                    }
                }
                DecompressorState::OutputMultiple(decode_buf, cur_idx, len) => {
                    let n_remaining = len - cur_idx;
                    if n_remaining <= buf.len() {
                        let pair = buf.split_at_mut(n_remaining);
                        pair.0.copy_from_slice(&decode_buf[cur_idx..len]);
                        buf = pair.1;
                        n += n_remaining;
                        self.decode_next()
                    } else {
                        let new_idx = cur_idx + buf.len();
                        n += buf.len();
                        buf.copy_from_slice(&decode_buf[cur_idx..new_idx]);
                        buf = &mut [];
                        Some(DecompressorState::OutputMultiple(decode_buf, new_idx, len))
                    }
                }
            }
        }

        Ok(n)
    }
}

#[derive(Debug)]
pub struct StreamDecompressor<R> {
    src: R,
    cur_byte: Option<u8>,
    cur_bit: u8,
    output_pos: usize,
    dict: Box<[u8]>,
    state: Option<DecompressorState>,
}

impl<R: ReadBytesExt> StreamDecompressor<R> {
    pub fn new(src: R) -> Self {
        let dict = vec![0u8; 0x2010].into();
        Self {
            src,
            cur_byte: None,
            cur_bit: 0x80,
            output_pos: 0,
            dict,
            state: Some(DecompressorState::Init),
        }
    }

    pub fn ensure_next_byte(&mut self) -> io::Result<Option<u8>> {
        if self.cur_byte.is_none() {
            self.cur_byte = match self.src.read_u8() {
                Ok(b) => Some(b),
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        None
                    } else {
                        return Err(e);
                    }
                }
            };
        }

        Ok(self.cur_byte)
    }

    fn next_bit(&mut self) -> io::Result<Option<bool>> {
        if let Some(b) = self.ensure_next_byte()? {
            let ret = (b & self.cur_bit) != 0;
            if self.cur_bit == 1 {
                self.cur_bit = 0x80;
                self.cur_byte = None;
            } else {
                self.cur_bit >>= 1;
            }

            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }

    fn next_bits<const N: usize>(&mut self) -> io::Result<Option<u16>> {
        debug_assert!(N <= 16);

        let mut result = 0;
        for _ in 0..N {
            if let Some(bit) = self.next_bit()? {
                result = (result << 1) | (bit as u16);
            } else {
                return Ok(None);
            }
        }

        Ok(Some(result))
    }

    fn decode_next(&mut self) -> io::Result<Option<DecompressorState>> {
        match self.next_bit()? {
            Some(true) => {
                if let Some(val) = self.next_bits::<8>()? {
                    let val = val as u8;
                    self.dict[self.output_pos & 0x1FFF] = val;
                    self.output_pos += 1;

                    Ok(Some(DecompressorState::OutputSingle(val)))
                } else {
                    Ok(None)
                }
            }
            Some(false) => {
                if let Some((idx, n)) = self.next_bits::<13>()?.zip(self.next_bits::<4>()?) {
                    let idx = idx.wrapping_sub(1) as usize;
                    let n = (n + 3) as usize;

                    assert!(n <= 19, "invalid decode length");

                    let mut decode_buf = [0; 19];
                    for (i, out) in decode_buf.iter_mut().take(n).enumerate() {
                        self.dict[self.output_pos & 0x1FFF] = self.dict[idx + i];
                        *out = self.dict[idx + i];
                        self.output_pos += 1;
                    }

                    Ok(Some(DecompressorState::OutputMultiple(decode_buf, 0, n)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl<R: ReadBytesExt> Read for StreamDecompressor<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        while let Some(state) = self.state.take() {
            if buf.is_empty() {
                self.state = Some(state);
                break;
            }

            self.state = match state {
                DecompressorState::Init => self.decode_next()?,
                DecompressorState::OutputSingle(val) => {
                    if let Some((next_out, rest)) = buf.split_first_mut() {
                        *next_out = val;
                        buf = rest;
                        n += 1;
                        self.decode_next()?
                    } else {
                        self.state = Some(DecompressorState::OutputSingle(val));
                        break;
                    }
                }
                DecompressorState::OutputMultiple(decode_buf, cur_idx, len) => {
                    let n_remaining = len - cur_idx;
                    if n_remaining <= buf.len() {
                        let pair = buf.split_at_mut(n_remaining);
                        pair.0.copy_from_slice(&decode_buf[cur_idx..len]);
                        buf = pair.1;
                        n += n_remaining;
                        self.decode_next()?
                    } else {
                        let new_idx = cur_idx + buf.len();
                        n += buf.len();
                        buf.copy_from_slice(&decode_buf[cur_idx..new_idx]);
                        buf = &mut [];
                        Some(DecompressorState::OutputMultiple(decode_buf, new_idx, len))
                    }
                }
            }
        }

        Ok(n)
    }
}
