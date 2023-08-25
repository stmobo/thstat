use std::io::Read;

#[derive(Debug)]
struct CryptState {
    key: u8,
    step: u8,
}

impl CryptState {
    const fn new(key: u8, step: u8) -> Self {
        Self { key, step }
    }

    fn decrypt_block(&mut self, input: &[u8], output: &mut [u8]) {
        assert_eq!(input.len(), output.len());
        assert_eq!(input.len() % 2, 0);
        assert!(input.len() > 4);

        /* Bytes from the first half are used to compute odd-indexed bytes in the output block.
         * Bytes from the second half correspond to even-indexed output bytes.
         */
        let (first, second) = input.split_at(input.len() / 2);
        for (i, half) in [first, second].into_iter().enumerate() {
            let mut out_idx = output.len() - 1 - i;

            for (j, in_byte) in half.iter().enumerate() {
                output[out_idx] = *in_byte ^ self.key;

                if j != (half.len() - 1) {
                    out_idx = out_idx.checked_sub(2).unwrap();
                }

                self.key = self.key.wrapping_add(self.step);
            }
        }
    }
}

#[derive(Debug)]
pub struct ThCrypt<R> {
    state: CryptState,
    out_buf: Vec<u8>,
    in_buf: Box<[u8]>,
    in_cursor: usize,
    src: R,
}

impl<R: Read> ThCrypt<R> {
    pub fn new(src: R, key: u8, step: u8, block_sz: usize) -> Self {
        assert!(block_sz >= 4);
        assert_eq!(block_sz % 2, 0);

        let buf = Vec::with_capacity(block_sz);
        let block_buf = vec![0u8; block_sz].into();
        Self {
            state: CryptState::new(key, step),
            src,
            out_buf: buf,
            in_buf: block_buf,
            in_cursor: 0,
        }
    }

    fn decrypt_next_block(&mut self) -> std::io::Result<bool> {
        let n = self.src.read(&mut self.in_buf[self.in_cursor..])?;

        if n == 0 {
            if self.in_cursor > 0 {
                if self.in_cursor < self.in_buf.len() / 4 {
                    /* just copy remaining bytes to output buffer */
                    self.out_buf.clear();
                    self.out_buf
                        .extend_from_slice(&self.in_buf[..self.in_cursor]);
                } else if self.in_cursor % 2 == 1 {
                    /* Decrypt everything but the last byte, which is simply copied */
                    self.out_buf.resize(self.in_cursor, 0);
                    let (last, block) = self.in_buf[..self.in_cursor].split_last().unwrap();
                    self.state
                        .decrypt_block(block, &mut self.out_buf[..self.in_cursor - 1]);
                    self.out_buf.push(*last);
                } else {
                    self.out_buf.resize(self.in_cursor, 0);
                    self.state.decrypt_block(
                        &self.in_buf[..self.in_cursor],
                        &mut self.out_buf[..self.in_cursor],
                    );
                }

                self.in_cursor = 0;
            }

            Ok(true)
        } else {
            self.in_cursor += n;
            if self.in_cursor >= self.in_buf.len() {
                self.out_buf.resize(self.in_buf.len(), 0);
                self.state
                    .decrypt_block(&self.in_buf[..], &mut self.out_buf[..]);
                self.in_cursor = 0;
            }

            Ok(false)
        }
    }

    fn drain_to_buf(&mut self, buf: &mut [u8]) -> usize {
        let amt = self.out_buf.len().min(buf.len());
        for (dst, src) in buf[..amt].iter_mut().zip(self.out_buf.drain(..amt)) {
            *dst = src;
        }
        amt
    }
}

impl<R: Read> Read for ThCrypt<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut n = 0;
        if !self.out_buf.is_empty() {
            n = self.drain_to_buf(buf);
            buf = &mut buf[n..];
        }

        while !buf.is_empty() {
            let done = self.decrypt_next_block()?;
            let next_amt = self.drain_to_buf(buf);

            n += next_amt;
            buf = &mut buf[n..];

            if done {
                break;
            }
        }

        Ok(n)
    }
}
