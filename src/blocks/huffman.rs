use sources::bitsource::BitSource;
use errors::{GzipResult, GzipError};

#[derive(Debug)]
pub struct HuffmanNode {
    code: Option<u16>,
    bit0: Option<Box<HuffmanNode>>,
    bit1: Option<Box<HuffmanNode>>
}

pub type Huffman = HuffmanNode;

type HuffmanCode = Vec<(u8, u16, u32)>;

impl HuffmanNode {
   pub fn build(codes : Vec<u8>) -> GzipResult<Self> {
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
        let valid_codes = codes.iter().enumerate().filter(|&(_, x)| *x > 0);
        for (code, bits) in valid_codes {
            let triple =
                (*bits, code as u16,
                 Self::reverse_bits(next_count[*bits as usize], *bits));
            huffman.push(triple);
            next_count[*bits as usize] += 1;
        }
        huffman.sort();
        println!("huffman {:?}", huffman);
        Self::build_trie(&huffman, 0, huffman.len() - 1, 1)
    }

    fn reverse_bits(value : u32, bits: u8) -> u32 {
        let mut ans = 0;
        let mut value = value;
        for _ in 0..bits {
            ans = (ans << 1) | (value & 1);
            value >>= 1;
        }
        ans
    }

    fn build_trie(huffman: &HuffmanCode,
                  start: usize, end: usize, mask: u32) -> GzipResult<Self> {
        if start == end {
            return Ok(HuffmanNode {
                code: Some(huffman[start].1),
                bit0: None,
                bit1: None
            });
        }
        let mut first = 0;
        for i in start..(end + 1) {
            if huffman[i].2 & mask > 0 {
                first = i;
                break;
            }
        }
        if first == 0 {
            return Err(GzipError::InternalError);
        }
        Ok(HuffmanNode {
            code: None,
            bit0: Some(Box::new(
                      Self::build_trie(huffman, start, first - 1, mask << 1)?)),
            bit1: Some(Box::new(
                      Self::build_trie(huffman, first, end, mask << 1)?))
        })
    }

    pub fn get_code(huffman: &Self, input: &mut BitSource) -> GzipResult<u16> {
        if let Some(code) = huffman.code {
            Ok(code)
        } else {
            let bit = input.get_bit()?;
            match bit {
                0 => {
                    let bit = huffman.bit0
                        .as_ref()
                        .ok_or(GzipError::InternalError)?;
                    Self::get_code(bit, input)
                },
                1 => {
                    let bit = huffman.bit1
                        .as_ref()
                        .ok_or(GzipError::InternalError)?;
                    Self::get_code(bit, input)
                }
                _ => Err(GzipError::InternalError)
            }
        }
    }
}
