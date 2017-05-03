use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use OutputBuffer;
use blocks::huffman::Huffman;
use blocks::window::{WindowDecoder, BlockWindow};
use context::VERBOSE;

#[allow(non_snake_case)]
struct DynamicHeader {
    HLIT: u16,
    HDIST: u16,
    HCLEN: u16
}

pub struct BlockDynamic<'a, T: 'a + BitSource> {
    input: &'a mut T,
    output: &'a mut OutputBuffer,
    literals: Option<Huffman>,
    distances: Option<Huffman>
}

impl<'a, T: BitSource> BlockWindow<'a, T> for BlockDynamic<'a, T> {
    fn get_input(&mut self) -> &mut T {
        self.input
    }

    fn get_output(&mut self) -> &mut OutputBuffer {
        self.output
    }
}

const CODE_LENGTHS_UNSHUFFLE : [usize; 19] =
    [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

impl<'a, T: BitSource> BlockDynamic<'a, T> {
    pub fn new(input: &'a mut T, output: &'a mut OutputBuffer) -> Self {
        BlockDynamic {
            input,
            output,
            literals: None,
            distances: None
        }
    }

    pub fn decode(&mut self) -> GzipResult<()> {
        let header = DynamicHeader {
            HLIT: 257 + self.input.get_bits_rev(5)? as u16,
            HDIST: 1 + self.input.get_bits_rev(5)? as u16,
            HCLEN: 4 + self.input.get_bits_rev(4)? as u16
        };
        verbose!(1, "Dynamic huffman block, HLIT {}, HDIST {}, HCLEN {}",
                 header.HLIT, header.HDIST, header.HCLEN);
        let mut code_lengths : Vec<u8> = vec![0; 19];
        for i in 0..header.HCLEN {
            let pos = CODE_LENGTHS_UNSHUFFLE[i as usize];
            code_lengths[pos] = self.input.get_bits_rev(3)? as u8;
        }
        let code_huffman = Huffman::build(code_lengths)?;
        let size = (header.HLIT + header.HDIST) as usize;
        let mut huff_lengths = self.decode_lengths(&code_huffman, size)?;
        self.distances = Some(Huffman::build(
            huff_lengths.split_off(header.HLIT as usize))?);
        self.literals = Some(Huffman::build(huff_lengths)?);
        self.window_decode()
    }

    fn decode_lengths(&mut self, code_huffman: &Huffman, size: usize)
        -> GzipResult<Vec<u8>> {

        let mut huff_lengths: Vec<u8> = vec![];
        let mut previous : Option<u8> = None;
        while huff_lengths.len() < size {
            match Huffman::get_code(&code_huffman, self.input)? as u8 {
                c @ 0...15 => {
                    huff_lengths.push(c);
                    previous = Some(c);
                },
                16 => {
                    let repeat = 3 + self.input.get_bits_rev(2)? as usize;
                    let value = previous
                        .ok_or(GzipError::InvalidDeflateStream)?;
                    huff_lengths.append(&mut vec![value; repeat]);
                },
                17 => {
                    let size = 3 + self.input.get_bits_rev(3)? as usize;
                    huff_lengths.append(&mut vec![0; size]);
                    previous = Some(0);
                },
                18 => {
                    let size = 11 + self.input.get_bits_rev(7)? as usize;
                    huff_lengths.append(&mut vec![0; size]);
                    previous = Some(0);
                },
                _ => return Err(GzipError::InternalError)
            }
        }
        Ok(huff_lengths)
    }
}

impl<'a, T: BitSource> WindowDecoder<'a, T> for BlockDynamic<'a, T> {
    fn get_literal(&mut self) -> GzipResult<u32> {
        let huffman = self.literals.as_ref().ok_or(GzipError::InternalError)?;
        Huffman::get_code(&huffman, self.input)
    }

    fn get_distance(&mut self) -> GzipResult<u32> {
        let huffman = self.distances.as_ref().ok_or(GzipError::InternalError)?;
        Huffman::get_code(&huffman, self.input)
    }
}

