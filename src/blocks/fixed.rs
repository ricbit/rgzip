use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use OutputBuffer;

pub struct BlockFixed<'a, 'b, T: 'a + BitSource, U: 'b + OutputBuffer> {
    input: &'a mut T,
    output: &'b mut U,
}

const LENGTH_EXTRA : [u8; 29] =
    [0, 0, 0, 0,
     0, 0, 0, 0,
     1, 1, 1, 1,
     2, 2, 2, 2,
     3, 3, 3, 3,
     4, 4, 4, 4,
     5, 5, 5, 5, 0];
const LENGTH_START : [u32; 29] =
    [3, 4, 5, 6,
     7, 8, 9, 10,
     11, 13, 15, 17,
     19, 23, 27, 31,
     35, 43, 51, 59,
     67, 83, 99, 115,
     131, 163, 195, 227, 258];

#[test]
fn ensure_lengths_are_consistent() {
    let n = 285 - 257 + 1;
    assert!(LENGTH_EXTRA.len() == n);
    assert!(LENGTH_START.len() == n);
    // We count only up to 227, because 258 has a double encoding.
    for i in 0..(LENGTH_EXTRA.len() - 2) {
        assert!(
            LENGTH_START[i + 1] == LENGTH_START[i] + (1 << LENGTH_EXTRA[i]));
    }
}

const DISTANCE_EXTRA : [u8; 30] =
    [0, 0, 0, 0,
     1, 1, 2, 2,
     3, 3, 4, 4,
     5, 5, 6, 6,
     7, 7, 8, 8,
     9, 9, 10, 10,
     11, 11, 12, 12,
     13, 13];

const DISTANCE_START : [u32; 30] =
    [1, 2, 3, 4,
     5, 7, 9, 13,
     17, 25, 33, 49,
     65, 97, 129, 193,
     257, 385, 513, 769,
     1025, 1537, 2049, 3073,
     4097, 6145, 8193, 12289,
     16385, 24577];

#[test]
fn ensure_distances_are_consistent() {
    for i in 0..(DISTANCE_EXTRA.len() - 1) {
        assert!(
            DISTANCE_START[i + 1] == 
                DISTANCE_START[i] + (1 << DISTANCE_EXTRA[i]));
    }
}

impl<'a, 'b, T: BitSource, U: OutputBuffer > BlockFixed<'a, 'b, T, U> {
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
                    let (length, distance) = self.get_window(code)?;
                    self.output.copy_window(distance, length)?;
                    println!("window {} {}", length, distance);
                    Ok(())
                },
                286...287 => Err(GzipError::InvalidDeflateStream),
                _ => Err(GzipError::InternalError),
            });
        }
    }

    fn get_window(&mut self, length_base: u32) -> GzipResult<(u32, u32)> {
        let index = length_base as usize - 257;
        let length = 
            LENGTH_START[index] + 
            self.input.get_bits_rev(LENGTH_EXTRA[index])?;
        let distance_base = self.input.get_bits(5)?;
        if distance_base >= 30 {
            return Err(GzipError::InvalidDeflateStream);
        }
        let index = distance_base as usize;
        let distance = 
            DISTANCE_START[index] + 
            self.input.get_bits_rev(DISTANCE_EXTRA[index])?;
        Ok((length, distance))
    }

    fn get_fixed(&mut self) -> GzipResult<u32> {
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
}


