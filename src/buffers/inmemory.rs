use errors::{GzipResult, GzipError};
use buffers::outputbuffer::OutputBuffer;
use sinks::bytesink::ByteSink;
use context::VERBOSE;

pub struct InMemoryBuffer<'a> {
    buffer: Vec<u8>,
    output: &'a mut ByteSink
}

impl<'a> InMemoryBuffer<'a> {
    pub fn new(output: &'a mut ByteSink) -> Self {
        InMemoryBuffer{ buffer: vec![], output }
    }
}

impl<'a> OutputBuffer for InMemoryBuffer<'a> {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer.push(data);
        self.output.put_u8(data)
    }

    fn put_data(&mut self, data: &mut Vec<u8>) -> GzipResult<()> {
        self.buffer.append(data);
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

