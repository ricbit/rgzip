use errors::GzipResult;
use sources::bytesource::ByteSource;
use sources::bitsource::BitSource;

pub struct BitAdapter<'a, T: 'a + ByteSource> {
    data: &'a mut T,
    pos: u8,
    cur: u8
}

impl<'a, T: ByteSource> BitAdapter<'a, T> {
    pub fn new(data: &'a mut T) -> Self {
        BitAdapter::<T>{ data, pos: 0, cur: 0 }
    }
}

impl<'a, T:ByteSource> BitSource for BitAdapter<'a, T> {
    fn get_bit(&mut self) -> GzipResult<u32> {
        if self.pos == 0 {
            self.cur = try!(self.data.get_u8());
            self.pos = 8;
        }
        let ans = self.cur & 1;
        self.cur >>= 1;
        self.pos -= 1;
        Ok(ans as u32)
    }
}

impl<'a, T:ByteSource> ByteSource for BitAdapter<'a, T> {
    fn get_u8(&mut self) -> GzipResult<u8> {
        self.pos = 0;
        self.data.get_u8()
    }
}


