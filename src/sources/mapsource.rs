use errors::{GzipResult, GzipError};
use sources::bytesource::ByteSource;
use memmap::{Mmap, Protection};

pub struct MapSource {
    file: Mmap,
    pos: usize
}

impl ByteSource for MapSource {
    fn get_u8(&mut self) -> GzipResult<u8> {
        let data = unsafe { self.file.as_slice() };
        let ans = if self.pos < data.len() {
            Ok(data[self.pos])
        } else {
            Err(GzipError::TruncatedFile)
        };
        self.pos += 1;
        ans
    }
}

impl MapSource {
    pub fn from_file(name: &str) -> GzipResult<Self> {
        use GzipError::*;

        let file = Mmap::open_path(name, Protection::Read)
            .or(Err(CantOpenFile))?;
        Ok(MapSource{ file, pos: 0 })
    }
}


