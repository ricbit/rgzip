use sources::bitsource::BitSource;
use errors::{GzipResult, GzipError};

#[derive(Debug)]
pub enum HuffmanNode {
    Code(u16),
    Node{bit0: Box<HuffmanNode>, bit1: Box<HuffmanNode>}
}

pub type Huffman = HuffmanNode;

type HuffmanCode = Vec<(u8, u16, u32)>;
type HuffmanSlice = [(u8, u16, u32)];

impl HuffmanNode {
    pub fn build(codes : Vec<u8>) -> GzipResult<Self> {
        let max = (1 + codes.iter().max().unwrap()) as usize;
        let mut bit_count = vec![0; max];
        for i in &codes {
            bit_count[*i as usize] += 1;
        }
        let mut next_count = vec![0; max];
        let mut prev = 0;
        for i in 1..max {
            next_count[i] = prev;
            prev = (prev + bit_count[i]) << 1;
        }
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
        Self::build_trie(&huffman, 0, huffman.len() - 1, 1)
    }

    pub fn print(tree: &Self, prefix: String) -> String {
        match *tree {
            HuffmanNode::Code(code) => {
                format!("{} : {}", prefix, code)
            }, 
            HuffmanNode::Node{ref bit0, ref bit1} => {
                format!("{}\n{}",
                    HuffmanNode::print(bit0.as_ref(), prefix.clone() + "0"),
                    HuffmanNode::print(bit1.as_ref(), prefix.clone() + "1"))
            }
        }
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

    fn build_trie(huffman: &HuffmanSlice,
                  start: usize, end: usize, mask: u32) -> GzipResult<Self> {
        if start == end {
            return Ok(HuffmanNode::Code(huffman[start].1));
        }
        let first = huffman[start..(end + 1)].iter().position(|x| x.2 & mask > 0).unwrap_or(0);
        if first == 0 {
            return Err(GzipError::InternalError);
        }
        Ok(HuffmanNode::Node{
            bit0: Box::new(
                      Self::build_trie(huffman, start, first - 1, mask << 1)?),
            bit1: Box::new(
                      Self::build_trie(huffman, first, end, mask << 1)?)
        })
    }

    pub fn get_code(huffman: &Self, input: &mut BitSource) -> GzipResult<u32> {
        match *huffman {
            HuffmanNode::Code(code) => Ok(code as u32),
            HuffmanNode::Node{ref bit0, ref bit1} => {
                let bit = input.get_bit()?;
                match bit {
                    0 => Self::get_code(bit0.as_ref(), input),
                    1 => Self::get_code(bit1.as_ref(), input),
                    _ => unreachable!()
                }
            }
        }
    }
}
