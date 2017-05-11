use sources::bitsource::BitSource;
use errors::{GzipResult, GzipError};

#[derive(Debug)]
pub enum HuffmanNode {
    Code(u16),
    Node{bits: u8, nodes: Vec<Box<HuffmanNode>>}
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
        Self::build_trie(&huffman, 0, huffman.len() - 1, 0)
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
                  start: usize, end: usize, right: u16) -> GzipResult<Self> {
        if start == end {
            return Ok(HuffmanNode::Code(huffman[start].1));
        }

        let smallest = huffman[start].0 as u16;
        let size = smallest - right;
        let mut nodes : Vec<(u32, Box<HuffmanNode>)> = vec![];
        let mask = ((1 << size) - 1) << right;

        for i in 0..(1 << size) {
            let reverse_i = Self::reverse_bits(i, size as u8) << right;
            let newstart = start + huffman[start..(end + 1)]
                .iter()
                .position(|x| x.2 & mask == reverse_i)
                .ok_or(GzipError::InternalError)?;
            let newend = newstart + huffman[newstart..(end + 1)]
                .iter()
                .position(|x| x.2 & mask != reverse_i)
                .unwrap_or(end - newstart + 1);
            nodes.push((i, Box::new(
                Self::build_trie(
                    huffman, newstart, newend - 1, right + size)?)));
        }
        let mut reversed : Vec<Box<HuffmanNode>>= vec![];
        for i in 0..(1 << size) {
            let reverse_i = Self::reverse_bits(i, size as u8);
            let index = nodes
                .iter()
                .position(|&(x, _)| x == reverse_i)
                .ok_or(GzipError::InternalError)?;
            let (_, node) = nodes.swap_remove(index as usize);
            reversed.push(node);
        }
        Ok(HuffmanNode::Node{ bits: size as u8, nodes: reversed })
    }

    pub fn get_code(huffman: &Self, input: &mut BitSource)
        -> GzipResult<u32> {

        let mut node = huffman;
        while let &HuffmanNode::Node{bits, ref nodes} = node {
            let bits = input.get_bits_rev(bits)?;
            node = nodes[bits as usize].as_ref();
        }
        if let &HuffmanNode::Code(code) = node {
            Ok(code as u32)
        } else {
            Err(GzipError::InternalError)
        }
    }
}
