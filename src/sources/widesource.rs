use std::io::prelude::*;
use std::fs::File;
use std::mem::transmute;
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;

pub struct WideSource {
    data : Vec<u8>,
    pos: usize
}

impl ByteSource for WideSource {
    fn get_u8(&mut self) -> GzipResult<u8> {
        let ans = if self.pos < self.data.len() {
            Ok(self.data[self.pos])
        } else {
            Err(GzipError::TruncatedFile)
        };
        self.pos += 1;
        ans
    }

    fn get_u64(&mut self) -> GzipResult<u64> {
        if self.pos + 8 < self.data.len() {
            let small = &self.data[self.pos..self.pos + 8];
            let ans = unsafe { transmute::<&[u8], &[u64]>(small)[0].to_le() };
            self.pos += 8;
            Ok(ans)
        } else {
            Err(GzipError::TruncatedFile)
        }
    }
}

impl WideSource {
    pub fn from_file(name: &str) -> GzipResult<Self> {
        use GzipError::*;
        let mut data = vec![];
        let mut file = File::open(name).or(Err(CantOpenFile))?;
        file.read_to_end(&mut data).or(Err(CantReadFile))?;
        Ok(WideSource{ data, pos: 0 })
    }
}


