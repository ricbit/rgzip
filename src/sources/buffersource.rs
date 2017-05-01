use std::io::prelude::*;
use std::fs::File;
use std::io::{Bytes, BufReader};
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;

pub struct BufferSource {
    file: Bytes<BufReader<File>>
}

impl ByteSource for BufferSource {
    fn get_u8(&mut self) -> GzipResult<u8> {
        match self.file.next() {
            Some(Ok(v)) => Ok(v),
            _ => Err(GzipError::TruncatedFile)
        }
    }
}

impl BufferSource {
    pub fn from_file(name: &String) -> GzipResult<Self> {
        use GzipError::*;
        let file = File::open(name).or(Err(CantOpenFile))?;
        Ok(BufferSource{ file: BufReader::new(file).bytes() })
    }
}


