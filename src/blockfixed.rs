use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use sinks::bytesink::ByteSink;

pub struct BlockFixed<'a, 'b, T: 'a + BitSource, U: 'b + ByteSink> {
    input: &'a mut T,
    output: &'b mut U,
}

impl<'a, 'b, T: BitSource, U: ByteSink> BlockFixed<'a, 'b, T, U> {
    pub fn new(input: &'a mut T, output: &'b mut U) -> Self {
        BlockFixed{ input: input, output: output }
    }

    pub fn decode(&'a mut self) -> GzipResult<()> {
        loop {
            let code = self.get_fixed()?;
            try!(match code {
                0...255 => {
                    self.output.put_u8(code as u8)?;
                    println!("letter {}", code as u8 as char);
                    Ok(())
                },
                256 => return Ok(()),
                257...285 =>  {
                    println!("window {}", code);
                    Ok(())
                },
                286...287 => Err(GzipError::InvalidDeflateStream),
                _ => Err(GzipError::InternalError),
            });
        }
    }

    fn get_fixed(&mut self) -> GzipResult<u32> {
        let base = self.input.get_bits(7)?;
        match base {
            0...0x17 => 
                Ok(base + 256),
            0x18...0x5F => 
                Ok((base << 1) + self.input.get_bits(1)?),
            0x60...0x63 => 
                Ok((base << 1) + self.input.get_bits(1)? + 88),
            0x64...0x7F => 
                Ok((base << 2) + self.input.get_bits(2)? - 256),
            _ => Err(GzipError::InternalError)
        }
    }
}


