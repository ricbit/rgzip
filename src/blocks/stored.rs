use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use OutputBuffer;
use context::VERBOSE;

#[allow(non_snake_case)]
struct StoredHeader {
    LEN: u16,
    NLEN: u16
}

pub struct BlockStored<'a, T: 'a + BitSource> {
    input: &'a mut T,
    output: &'a mut OutputBuffer,
}

impl<'a, T: BitSource> BlockStored<'a, T> {
    pub fn new(input: &'a mut T, output: &'a mut OutputBuffer) -> Self {
        BlockStored{ input, output }
    }

    pub fn decode(&mut self) -> GzipResult<()> {
        let header = StoredHeader{
            LEN: self.input.get_u16()?,
            NLEN: self.input.get_u16()?
        };
        if header.LEN ^ header.NLEN != 65535 {
            return Err(GzipError::StoredHeaderFailure);
        }
        for _ in 0..header.LEN {
            let byte = self.input.get_u8()?;
            self.output.put_u8(byte)?;
        }
        verbose!(1, "Stored block, len = {}", header.LEN);
        Ok(())
    }
}


