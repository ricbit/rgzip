use std::io::prelude::*;
use std::fs::File;
use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;

pub struct VecBufSource {
    file: File,
    data: Vec<u8>,
    pos: usize,
    size: usize
}

const SIZE : usize = 32768;

impl ByteSource for VecBufSource {
    fn get_u8(&mut self) -> GzipResult<u8> {
        if self.pos >= self.size {
            self.size = self.file
                .read(&mut self.data)
                .or(Err(GzipError::TruncatedFile))?;
            self.pos = 0;
        }
        let ans = self.data[self.pos];
        self.pos += 1;
        Ok(ans)
    }
}

impl VecBufSource {
    pub fn from_file(name: &String) -> GzipResult<Self> {
        use GzipError::*;
        let data = vec![0; SIZE];
        let file = File::open(name).or(Err(CantOpenFile))?;
        Ok(VecBufSource{ file, data, pos: 0, size: 0 })
    }
}



