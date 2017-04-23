use errors::GzipError;

pub trait ByteSource {
    fn get_u8(&mut self) -> Result<u8, GzipError>;

    fn get_variable(&mut self, size: u8) -> Result<u32, GzipError> {
        let mut ans : u32 = 0;
        for i in 0..size {
            ans |= (try!(self.get_u8()) as u32) << (8 * i);
        }
        Ok(ans)
    }

    fn get_u16(&mut self) -> Result<u16, GzipError> {
        self.get_variable(2).map(|x| x as u16)
    }

    fn get_u32(&mut self) -> Result<u32, GzipError> {
        self.get_variable(4)
    }
}


