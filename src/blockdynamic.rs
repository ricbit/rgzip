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

const CODE_LENGTHS_UNSHUFFLE : [usize; 19] =
    [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

#[derive(Debug)]
struct HuffmanNode {
    code: Option<u16>,
    bit0: Option<Box<HuffmanNode>>,
    bit1: Option<Box<HuffmanNode>>
}

type HuffmanCode = Vec<(u8, u16, u32)>;

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
            let pos = CODE_LENGTHS_UNSHUFFLE[i as usize];
            code_lengths[pos] = self.input.get_bits_rev(3)? as u8;
        }
        println!("code len {:?}", code_lengths);
        let code_huffman = self.build_huffman(code_lengths);
        println!("code huff {:?}", code_huffman);

        Ok(())
    }

    fn reverse_bits(&self, value : u32, bits: u8) -> u32 {
        let mut ans = 0;
        let mut value = value;
        for _ in 0..bits {
            ans = (ans << 1) | (value & 1);
            value >>= 1;
        }
        ans
    }

    fn build_huffman(&self, codes : Vec<u8>) -> HuffmanNode {
        let max = (1 + codes.iter().max().unwrap()) as usize;
        let mut bit_count = vec![0; max];
        for i in &codes {
            bit_count[*i as usize] += 1;
        }
        println!("bit count {:?}", bit_count);
        let mut next_count = vec![0; max];
        let mut prev = 0;
        for i in 1..max {
            next_count[i] = prev;
            prev = (prev + bit_count[i]) << 1;
        }
        println!("next_count {:?}", next_count);
        let mut huffman : HuffmanCode = vec![];
        for (code, bits) in codes.iter().enumerate().filter(|&(i,x)| *x > 0) {
            let triple =
                (*bits, code as u16,
                 self.reverse_bits(next_count[*bits as usize], *bits));
            huffman.push(triple);
            next_count[*bits as usize] += 1;
        }
        huffman.sort();
        println!("huffman {:?}", huffman);
        self.build_trie(&huffman, 0, huffman.len() - 1, 1)
    }

    fn build_trie(&self, huffman: &HuffmanCode,
                  start: usize, end: usize, mask: u32) -> HuffmanNode {
        if start == end {
            return HuffmanNode {
                code: Some(huffman[start].1),
                bit0: None,
                bit1: None
            };
        }
        let mut first = 0;
        for i in start..(end + 1) {
            if huffman[i].2 & mask > 0 {
                first = i;
                break;
            }
        }
        HuffmanNode {
            code: None,
            bit0: Some(Box::new(
                      self.build_trie(huffman, start, first - 1, mask << 1))),
            bit1: Some(Box::new(
                      self.build_trie(huffman, first, end, mask << 1)))
        }
    }
}


