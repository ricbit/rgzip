use errors::{GzipResult, GzipError};
use buffers::outputbuffer::OutputBuffer;
use sinks::bytesink::{ByteSink, ByteSinkProvider};
use std::ptr;

pub struct CopyBuffer {
    buffer: Vec<u8>,
    pos: usize,
    size: usize,
    output: Box<ByteSink>
}

impl CopyBuffer {
    pub fn new(provider: ByteSinkProvider) -> GzipResult<Self> {
        let output = provider()?;
        Ok(CopyBuffer{ buffer: vec![0; 32768], pos: 0, size: 0, output })
    }
}

impl OutputBuffer for CopyBuffer {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer[self.pos] = data;
        self.size += 1;
        self.pos += 1;
        if self.pos >= 32768 {
            self.pos = 0;
            self.output.put_data(&self.buffer)?;
        }
        Ok(())
    }

    fn copy_window(&mut self, distance: u32, length: u32) -> GzipResult<()> {
        let distance = distance as usize;
        let length = length as usize;
        if distance > self.size {
            return Err(GzipError::InvalidDeflateStream);
        }
        if self.pos >= distance &&
            self.pos + length < 32767 &&
            distance > length {

            let begin = self.buffer.as_mut_ptr();
            unsafe {
                ptr::copy_nonoverlapping(
                    begin.offset((self.pos - distance) as isize),
                    begin.offset(self.pos as isize),
                    length);
            }
            self.pos += length;
        } else {
            let index : usize = self.pos + 32768 - distance;
            for i in 0..length {
                let data = self.buffer[(index + i) & 32767];
                self.buffer[self.pos] = data;
                self.pos += 1;
                if self.pos >= 32768 {
                    self.pos = 0;
                    self.output.put_data(&self.buffer)?;
                }
            }
        }
        self.size += length as usize;
        Ok(())
    }
}

impl Drop for CopyBuffer {
    fn drop(&mut self) {
        self.output.put_data(&self.buffer[0..self.pos]).unwrap();
    }
}
