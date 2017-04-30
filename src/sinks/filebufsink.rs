use std::fs::File;
use std::io::Write;
use errors::{GzipResult, GzipError};
use sinks::bytesink::ByteSink;

pub struct FileBufSink {
    file: File,
    buffer: Vec<u8>,
    pos: usize
}

impl FileBufSink {
    pub fn new(name: &String) -> GzipResult<Self> {
        let file = File::create(name).or(Err(GzipError::CantCreateFile))?;
        Ok(FileBufSink{ file : file, buffer: vec![0; 32768], pos: 0 })
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
        if self.pos == 32768 {
            let limit = self.pos;
            self.flush(limit)
        } else {
            Ok(())
        }
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


