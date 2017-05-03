use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use blocks::window::{WindowDecoder, BlockWindow};
use OutputBuffer;
use context::VERBOSE;

pub struct BlockFixed<'a, T: 'a + BitSource> {
    input: &'a mut T,
    output: &'a mut OutputBuffer,
}

impl<'a, T: BitSource> BlockFixed<'a, T> {
    pub fn new(input: &'a mut T, output: &'a mut OutputBuffer) -> Self {
        BlockFixed{ input, output }
    }

    pub fn decode(&mut self) -> GzipResult<()> {
        verbose!(1, "Fixed huffman block");
        self.window_decode()
    }
}

impl<'a, T: BitSource> BlockWindow<'a, T> for BlockFixed<'a, T> {
    fn get_input(&mut self) -> &mut T {
        self.input
    }

    fn get_output(&mut self) -> &mut OutputBuffer {
        self.output
    }
}

impl<'a, T: BitSource> WindowDecoder<'a, T> for BlockFixed<'a, T> {
    fn get_literal(&mut self) -> GzipResult<u32> {
        let base = self.input.get_bits(7)?;
        match base {
            0...0x17 =>
                Ok(base + 256),
            0x18...0x5F =>
                Ok((base << 1) + self.input.get_bits(1)? - 0x30),
            0x60...0x63 =>
                Ok((base << 1) + self.input.get_bits(1)? - 0xC0 + 256),
            0x64...0x7F =>
                Ok((base << 2) + self.input.get_bits(2)? - 0x190 + 144),
            _ => Err(GzipError::InternalError)
        }
    }

    fn get_distance(&mut self) -> GzipResult<u32> {
        self.input.get_bits(5)
    }
}


