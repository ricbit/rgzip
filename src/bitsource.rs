use errors::GzipResult;
use bytesource::ByteSource;

pub trait BitSource : ByteSource {
    fn get_bit(&mut self) -> GzipResult<u32>;

    fn get_bits(&mut self, size: u8) -> GzipResult<u32> {
        let mut ans : u32 = 0;
        for _ in 0..size {
            ans = (ans << 1) | try!(self.get_bit());
        }
        Ok(ans)
    }

    fn get_bits_rev(&mut self, size: u8) -> GzipResult<u32> {
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= try!(self.get_bit()) << i;
        }
        Ok(ans)
    }
}


