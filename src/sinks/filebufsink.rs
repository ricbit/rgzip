use std::fs::File;
use std::io::Write;
use errors::{GzipResult, GzipError};
use sinks::bytesink::ByteSink;

pub struct FileBufSink {
    file: File,
    buffer: Vec<u8>,
    pos: usize
}

const BUFSIZE : usize = 32768;

impl FileBufSink {
    pub fn new(name: &String) -> GzipResult<Self> {
        let file = File::create(name).or(Err(GzipError::CantCreateFile))?;
        Ok(FileBufSink{ file : file, buffer: vec![0; BUFSIZE], pos: 0 })
    }

    pub fn flush(&mut self, limit: usize) -> GzipResult<()> {
        self.pos = 0;
        // TODO: check how many bytes actually got written.
        match self.file.write(&self.buffer[0..limit]) {
            Ok(0) | Err(_) => Err(GzipError::CantWriteFile),
            _ => Ok(())
        }
    }
}

impl ByteSink for FileBufSink {
    fn put_u8(&mut self, data: u8) -> GzipResult<()> {
        self.buffer[self.pos] = data;
        self.pos += 1;
        if self.pos == BUFSIZE {
            let limit = self.pos;
            self.flush(limit)
        } else {
            Ok(())
        }
    }

    fn put_data(&mut self, data: &mut Vec<u8>) -> GzipResult<()> {
        let left = BUFSIZE - self.pos;
        if data.len() <= left {
            self.buffer[self.pos..self.pos+data.len()].copy_from_slice(data);
            self.pos += data.len();
            if self.pos == BUFSIZE {
                self.flush(BUFSIZE)?;
            }
        } else {
            self.buffer[self.pos..BUFSIZE].copy_from_slice(&data[0..left]);
            self.flush(BUFSIZE)?;
            self.buffer[0..data.len()-left]
                .copy_from_slice(&data[left..data.len()]);
            self.pos = data.len() - left;
        }
        Ok(())
    }
}

impl Drop for FileBufSink {
    fn drop(&mut self) {
        let limit = self.pos;
        if self.flush(limit).is_err() {
            panic!("Could not write file");
        }
    }
}


