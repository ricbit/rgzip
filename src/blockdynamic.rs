use errors::{GzipResult, GzipError};
use sources::bitsource::BitSource;
use OutputBuffer;

#[allow(non_snake_case)]
struct DynamicHeader {
    HLIT: u16,
    HDIST: u16,
    HCLEN: u16
}

pub struct BlockDynamic<'a, 'b, T: 'a + BitSource, U: 'b + OutputBuffer> {
    input: &'a mut T,
    output: &'b mut U,
}

const code_lengths_unshuffle : [usize; 19] = 
    [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

impl<'a, 'b, T: BitSource, U: OutputBuffer> BlockDynamic<'a, 'b, T, U> {
    pub fn new(input: &'a mut T, output: &'b mut U) -> Self {
        BlockDynamic{ input: input, output: output }
    }

    pub fn decode(&'a mut self) -> GzipResult<()> {
        let header = DynamicHeader {
            HLIT: 257 + self.input.get_bits_rev(5)? as u16,
            HDIST: 1 + self.input.get_bits_rev(5)? as u16,
            HCLEN: 4 + self.input.get_bits_rev(4)? as u16
        };
        println!("Dynamic block, HLIT {}, HDIST {}, HCLEN {}", 
                 header.HLIT, header.HDIST, header.HCLEN);
        let mut code_lengths : Vec<u8> = vec![0; 19];
        for i in 0..header.HCLEN {
            let pos = code_lengths_unshuffle[i as usize];
            code_lengths[pos] = self.input.get_bits_rev(3)? as u8;
        }
        println!("code len {:?}", code_lengths);
        let code_huffman = self.build_huffman(code_lengths);
        println!("code huff {:?}", code_huffman);

        Ok(())
    }

    fn build_huffman(&self, codes : Vec<u8>) {
        let max = (1 + codes.iter().max().unwrap()) as usize;
        let mut bit_count = vec![0; max];
        for i in codes {
            bit_count[i as usize] += 1;
        }
        println!("bit count {:?}", bit_count);
        let mut next_count = vec![0; max];
        let mut prev = 0;
        for i in 1..max {
            next_count[i] = prev;
            prev = (prev + bit_count[i]) << 1;
        }
        println!("next_count {:?}", next_count);
    }
}


