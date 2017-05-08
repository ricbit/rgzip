use errors::GzipResult;

macro_rules! get_variable {
    ($self: tt, $type : ty, $size : expr) => {{
        let mut ans : $type = 0;
        for i in 0..$size {
            ans |= ($self.get_u8()? as $type) << (8 * i);
        }
        Ok(ans)
    }}
}

pub trait ByteSource {
    fn get_u8(&mut self) -> GzipResult<u8>;

    fn get_u16(&mut self) -> GzipResult<u16> {
        get_variable!(self, u16, 2)
    }

    fn get_u32(&mut self) -> GzipResult<u32> {
        get_variable!(self, u32, 4)
    }

    fn get_u64(&mut self) -> GzipResult<u64> {
        get_variable!(self, u64, 8)
    }
}


