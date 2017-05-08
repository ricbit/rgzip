use errors::GzipResult;
use sources::bytesource::ByteSource;
use sources::bitsource::BitSource;

pub struct BitAdapter<'a> {
    data: &'a mut ByteSource,
    pos: u8,
    cur: u64
}

impl<'a> BitAdapter<'a> {
    pub fn new(data: &'a mut ByteSource) -> Self {
        BitAdapter{ data, pos: 0, cur: 0 }
    }
}

impl<'a> BitSource for BitAdapter<'a> {
    fn get_bit(&mut self) -> GzipResult<u32> {
        if self.pos == 0 {
            self.cur = try!(self.data.get_u8()) as u64;
            self.pos = 8;
        }
        let ans = self.cur & 1;
        self.cur >>= 1;
        self.pos -= 1;
        Ok(ans as u32)
    }
}

impl<'a> ByteSource for BitAdapter<'a> {
    fn get_u8(&mut self) -> GzipResult<u8> {
        self.pos = 0;
        self.data.get_u8()
    }
}


