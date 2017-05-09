use std::io::prelude::*;
use std::fs::File;
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;

pub struct VecSource {
    data : Vec<u8>,
    pos: usize
}

impl ByteSource for VecSource {
    fn get_u8(&mut self) -> GzipResult<u8> {
        let ans = if self.pos < self.data.len() {
            Ok(self.data[self.pos])
        } else {
            Err(GzipError::TruncatedFile)
        };
        self.pos += 1;
        ans
    }
}

impl VecSource {
    pub fn from_file(name: &str) -> GzipResult<Self> {
        use GzipError::*;
        let mut data = vec![];
        let mut file = File::open(name).or(Err(CantOpenFile))?;
        file.read_to_end(&mut data).or(Err(CantReadFile))?;
        Ok(VecSource{ data, pos: 0 })
    }
}


