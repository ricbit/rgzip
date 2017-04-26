use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use sinks::bytesink::ByteSink;

#[allow(non_snake_case)]
struct StoredHeader {
    LEN: u16,
    NLEN: u16
}

pub struct BlockStored<'a, 'b, T: 'a + BitSource, U: 'b + ByteSink> {
    input: &'a mut T,
    output: &'b mut U,
}

impl<'a, 'b, T: BitSource, U: ByteSink> BlockStored<'a, 'b, T, U> {
    pub fn new(input: &'a mut T, output: &'b mut U) -> Self {
        BlockStored{ input: input, output: output }
    }

    pub fn decode(&'a mut self) -> GzipResult<()> {
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
        println!("Stored block, len = {}", header.LEN);
        Ok(())
    }
}


