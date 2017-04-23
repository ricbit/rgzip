use errors::GzipResult;

pub trait ByteSource {
    fn get_u8(&mut self) -> GzipResult<u8>;

    fn get_variable(&mut self, size: u8) -> GzipResult<u32> {
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= (try!(self.get_u8()) as u32) << (8 * i);
        }
        Ok(ans)
    }

    fn get_u16(&mut self) -> GzipResult<u16> {
        self.get_variable(2).map(|x| x as u16)
    }

    fn get_u32(&mut self) -> GzipResult<u32> {
        self.get_variable(4)
    }
}


