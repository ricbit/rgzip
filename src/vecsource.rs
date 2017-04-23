use std::io::prelude::*;
use std::fs::File;
use errors::GzipError;
use bytesource::ByteSource;

pub struct VecSource {
    data : Vec<u8>,
    pos: usize
}

impl ByteSource for VecSource {
    fn get_u8(&mut self) -> Result<u8, GzipError> {
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
    pub fn from_file(name: &String) -> Result<Self, GzipError> {
        use GzipError::*;
        let mut data = vec![];
        let mut file = try!(File::open(name).or(Err(CantOpenFile)));
        try!(file.read_to_end(&mut data).or(Err(CantReadFile)));
        Ok(VecSource{ data: data, pos: 0 })
    }
}


