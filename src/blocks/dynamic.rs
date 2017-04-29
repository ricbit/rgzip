use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use OutputBuffer;
use blocks::huffman::Huffman;

#[allow(non_snake_case)]
struct DynamicHeader {
    HLIT: u16,
    HDIST: u16,
    HCLEN: u16
}

pub struct BlockDynamic<'a, T: 'a + BitSource, U: 'a + OutputBuffer> {
    input: &'a mut T,
    output: &'a mut U,
}

const CODE_LENGTHS_UNSHUFFLE : [usize; 19] =
    [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

impl<'a, T: BitSource, U: OutputBuffer> BlockDynamic<'a, T, U> {
    pub fn new(input: &'a mut T, output: &'a mut U) -> Self {
        BlockDynamic{ input: input, output: output }
    }

    pub fn decode(&mut self) -> GzipResult<()> {
        let header = DynamicHeader {
            HLIT: 257 + self.input.get_bits_rev(5)? as u16,
            HDIST: 1 + self.input.get_bits_rev(5)? as u16,
            HCLEN: 4 + self.input.get_bits_rev(4)? as u16
        };
        println!("Dynamic block, HLIT {}, HDIST {}, HCLEN {}",
                 header.HLIT, header.HDIST, header.HCLEN);
        let mut code_lengths :Vec<u8> = vec![0; 19];
        for i in 0..header.HCLEN {
            let pos = CODE_LENGTHS_UNSHUFFLE[i as usize];
            code_lengths[pos] = self.input.get_bits_rev(3)? as u8;
        }
        println!("code len {:?}", code_lengths);
        let code_huffman = Huffman::build(code_lengths)?;
        //println!("code huff {:?}", code_huffman);
        let size = (header.HLIT + header.HDIST) as usize;
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
                    if previous == None {
                        return Err(GzipError::InvalidDeflateStream);
                    }
                    huff_lengths.append(&mut vec![previous.unwrap(); repeat]);
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
        /*println!("Huff len:");
        for (i, value) in huff_lengths.iter().enumerate() {
            println!("code {} => length {}", i, value);
        }*/
        Ok(())
    }

}


