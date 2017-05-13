use errors::GzipResult;
use sources::bytesource::ByteSource;
use sources::bitsource::BitSource;

pub struct WideAdapter<'a> {
    data: &'a mut ByteSource,
    pos: u8,
    cur: u64
}

impl<'a> WideAdapter<'a> {
    pub fn new(data: &'a mut ByteSource) -> Self {
        WideAdapter{ data, pos: 0, cur: 0 }
    }
}

impl<'a> BitSource for WideAdapter<'a> {
    fn get_bit(&mut self) -> GzipResult<u32> {
        if self.pos == 0 {
            if let Ok(data) = self.data.get_u64() {
                self.cur = data;
                self.pos = 64;
            } else {
                self.cur = self.data.get_u8()? as u64;
                self.pos = 8;
            }
        }
        let ans = self.cur & 1;
        self.cur >>= 1;
        self.pos -= 1;
        Ok(ans as u32)
    }

    fn get_bits_rev(&mut self, size: u8) -> GzipResult<u32> {
        if size <= self.pos {
            let ans = self.cur & ((1 << size) - 1);
            self.cur >>= size;
            self.pos -= size;
            Ok(ans as u32)
        } else {
            self.get_bits_rev_slow(size)
        }
    }
}

impl<'a> WideAdapter<'a> {
    fn get_bits_rev_slow(&mut self, size: u8) -> GzipResult<u32> {
        if self.pos > 0 {
            if let Ok(data) = self.data.get_u64() {
                let mut ans = self.cur;
                let left = size - self.pos;
                ans |= (data & ((1 << left) - 1)) << self.pos;
                self.cur = data >> left;
                self.pos = 64 - left;
                return Ok(ans as u32);
            }
        }
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= self.get_bit()? << i;
        }
        Ok(ans)
    }
}

impl<'a> ByteSource for WideAdapter<'a> {
    fn get_u8(&mut self) -> GzipResult<u8> {
        self.pos = 0;
        self.data.get_u8()
    }
}


