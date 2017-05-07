use errors::{GzipResult, GzipError};
use buffers::outputbuffer::OutputBuffer;
use sinks::bytesink::{ByteSink, ByteSinkProvider};
use context::VERBOSE;

pub struct InMemoryBuffer {
    buffer: Vec<u8>,
    output: Box<ByteSink>
}

impl InMemoryBuffer {
    pub fn new(provider: ByteSinkProvider) -> GzipResult<Self> {
        let output = provider()?;
        Ok(InMemoryBuffer{ buffer: vec![], output })
    }
}

impl OutputBuffer for InMemoryBuffer {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer.push(data);
        self.output.put_u8(data)
    }

    fn put_data(&mut self, data: Vec<u8>) -> GzipResult<()> {
        self.buffer.extend_from_slice(&data);
        self.output.put_data(data)
    }

    fn copy_window(&mut self, distance: u32, length: u32) -> GzipResult<()> {
        if distance as usize > self.buffer.len() {
            return Err(GzipError::InvalidDeflateStream);
        }
        let index : usize = self.buffer.len() - distance as usize;
        verbose!(2, "window char: ");
        for i in 0..length {
            let data = self.buffer[index + i as usize];
            verbose!(2, "-- {}", data as u8 as char);
            self.output.put_u8(data)?;
            self.buffer.push(data);
        }
        Ok(())
    }
}

