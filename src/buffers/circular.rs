use errors::{GzipResult, GzipError};
use buffers::outputbuffer::OutputBuffer;
use sinks::bytesink::{ByteSink, ByteSinkProvider};
use context::VERBOSE;

pub struct CircularBuffer {
    buffer: Vec<u8>,
    pos: usize,
    size: usize,
    output: Box<ByteSink>
}

impl CircularBuffer {
    pub fn new(provider: ByteSinkProvider) -> GzipResult<Self> {
        let output = provider()?;
        Ok(CircularBuffer{ buffer: vec![0; 32768], pos: 0, size: 0, output })
    }
}

impl OutputBuffer for CircularBuffer {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer[self.pos] = data;
        self.pos = (self.pos + 1) & 32767;
        self.size += 1;
        self.output.put_u8(data)
    }

    fn put_data(&mut self, data: Vec<u8>) -> GzipResult<()> {
        for d in data.iter() {
            self.buffer[self.pos] = *d;
            self.pos = (self.pos + 1) & 32767;
        }
        self.size += data.len();
        self.output.put_data(data)
    }

    fn copy_window(&mut self, distance: u32, length: u32) -> GzipResult<()> {
        let distance = distance as usize;
        if distance > self.size {
            println!("d {} {}", distance, self.size);
            return Err(GzipError::InvalidDeflateStream);
        }
        let index : usize = self.pos + 32768 - distance;
        verbose!(2, "window char: ");
        for i in 0..length {
            let data = self.buffer[(index + i as usize) & 32767];
            verbose!(2, "-- {}", data as u8 as char);
            self.buffer[self.pos] = data;
            self.pos = (self.pos + 1) & 32767;
            self.output.put_u8(data)?;
        }
        self.size += length as usize;
        Ok(())
    }
}


