use std::{collections::HashMap, io};
use std::hash::Hash;
use std::cmp::Reverse;
use bit_vec::BitVec;
use bitstream::{BitReader, BitWriter, NoPadding};
use std::path::Path;
use std::fs::File;

enum NodeType<X> {
    Internal (Box<HuffmanNode<X>>, Box<HuffmanNode<X>>),
    Leaf(X)
}

pub struct HuffmanNode<X> {
    pub weight: usize,
    node_type: NodeType<X>
}

type EncodeDict<X> = HashMap<X, BitVec>;

fn gen_entries<X>(node: HuffmanNode<X>, prefix: BitVec) -> EncodeDict<X> where X: Eq, X: Hash {
    let mut dic: EncodeDict<X> = HashMap::new();
    match node.node_type {
        NodeType::Leaf(sym) => { dic.insert(sym, prefix); dic }
        NodeType::Internal(node_a, node_b) => {
            let mut prefix_a = prefix.clone();
            prefix_a.push(false);
            dic.extend( gen_entries(*node_a, prefix_a) );
            let mut prefix_b = prefix.clone();
            prefix_b.push(true);
            dic.extend( gen_entries(*node_b, prefix_b) );
            dic
        }
    }
}

pub trait SerializeBytes {
    fn serialize_to_bytes(&self) -> Vec<u8>;
    fn sfb(bytes: &[u8]) -> Self;
    fn bitlen() -> usize;
}

impl SerializeBytes for u8 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
    fn sfb(bytes: &[u8]) -> Self {
        bytes[0]
    }
    fn bitlen() -> usize { 8 }
}

impl SerializeBytes for u16 {
    fn serialize_to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn sfb(bytes: &[u8]) -> Self {
        Self::from_le_bytes([bytes[0], bytes[1]])
    }
    fn bitlen() -> usize { 16 }
}

impl<X> HuffmanNode<X> {
    pub fn new(a: HuffmanNode<X>, b: HuffmanNode<X>) -> Self {
        Self { weight: a.weight + b.weight , node_type: NodeType::Internal(Box::new(a), Box::new(b))}
    }

    pub fn from_weights(weights: HashMap<X, usize>) -> Self where X: Eq, X: Hash, X: Ord, X: Copy {
        let mut occuring: Vec<(X, usize)> = weights.into_iter().filter(|(_, weight)| *weight!=0).collect();
        occuring.sort_by_key(|(sym, _)| *sym); //(collect->filter->)sort(->into_iter) is required to make the tree deterministic
        let mut nodes: Vec<HuffmanNode<X>> = occuring.into_iter().map(|(sym, weight)| HuffmanNode { weight, node_type: NodeType::Leaf(sym) }).collect();
        loop {
            nodes.sort_by_key(|f| Reverse(f.weight));
            let a = nodes.pop().unwrap();
            let b = nodes.pop().unwrap();
            let new_node = HuffmanNode::new(a, b);
            if nodes.is_empty() {
                return new_node;
            }
            nodes.push(new_node);
        }
    }

    pub fn to_dictionary(self) -> EncodeDict<X> where X: Eq, X: Hash {
        gen_entries(self, BitVec::new())
    }

    fn to_bitnode(&self) -> BitVec where X: SerializeBytes {
        let mut bv = BitVec::new();
        match &self.node_type {
            NodeType::Leaf(symbol) => {
                bv.push(true);
                bv.extend( BitVec::from_bytes(&symbol.serialize_to_bytes()) );
            }
            NodeType::Internal(_, _) => {
                bv.push(false);
                bv.extend( self.to_bits() );
            }
        }
        bv
    }

    fn to_bits(&self) -> BitVec where X: SerializeBytes {
        let mut bv = BitVec::new();
        match &self.node_type {
            NodeType::Internal(node_a, node_b) => {
                bv.extend( node_a.to_bitnode() );
                bv.extend( node_b.to_bitnode() );
                bv
            },
            _ => panic!("this node should not be a leaf")
        }
    }

    fn from_bitnode<R>(br: &mut BitReader<R, NoPadding>, readnleaf: usize) -> Option<(Self, &mut BitReader<R, NoPadding>)> where X: SerializeBytes, R: std::io::Read {
        match br.next() {
            Some(true) => {
                let mut bv = BitVec::new();
                for _ in 0..readnleaf {
                    match br.next() {
                        Some(bit) => bv.push(bit),
                        None => return None
                    }
                }
                let symbol = X::sfb( &bv.to_bytes() );
                Some(( Self {weight: 0, node_type: NodeType::Leaf(symbol)}, br ))
            }
            Some(false) => {
                let x = Self::from_bits(br, readnleaf)?;
                Some(x)
            }
            None => None
        }
    }

    fn from_bits<R>(br: &mut BitReader<R, NoPadding>, readnleaf: usize) -> Option<(Self, &mut BitReader<R, NoPadding>)> where X: SerializeBytes, R: std::io::Read {
        let (node_a, ba) = Self::from_bitnode(br, readnleaf)?;
        let (node_b, bb) = Self::from_bitnode(ba, readnleaf)?;
        Some(( Self {weight: 0, node_type: NodeType::Internal(Box::new(node_a), Box::new(node_b))}, bb ))
    }

    pub fn to_file<P>(&self, filename: P) -> io::Result<()> where X: SerializeBytes, X: std::fmt::Debug, P: AsRef<Path> {
        let mut file = File::create(filename)?;
        let mut bw = BitWriter::new(&mut file);

        let bv = self.to_bits();
        for bit in bv {
            bw.write_bit(bit)?;
        }
        Ok(())
    }

    pub fn from_file<P>(filename: P) -> io::Result<Self> where X: SerializeBytes, X: std::fmt::Debug, P: AsRef<Path> {
        let file = File::open(filename)?;
        let mut br = BitReader::new(&file);

        let (hufftree, _) = Self::from_bits(&mut br, X::bitlen()).unwrap();
        Ok( hufftree )
    }
}

pub fn count_freqs<X>(contents: X) -> HashMap<X::Item, usize> where X: Iterator, X::Item: Eq, X::Item: Hash {
    let mut counters = HashMap::new();
    for symbol in contents {
        let location = counters.entry(symbol).or_insert(0);
        *location += 1;
    }
    counters
}

pub fn encode<X>(input: &[X], dic: EncodeDict<X>) -> Vec<u8> where X: Eq, X: PartialEq, X: Hash {
    let mut encoded: Vec<u8> = Vec::new();
    let mut bw = BitWriter::new(&mut encoded);
    for symbol in input {
        let code = dic.get(symbol).expect("symbol should be in dictionary");
        for bit in code {
            bw.write_bit(bit).unwrap();
        }
    }
    drop(bw);
    encoded
}

fn get_internals<X>(root_node: HuffmanNode<X>) -> (HuffmanNode<X>, HuffmanNode<X>) {
    match root_node.node_type {
        NodeType::Internal(node_a, node_b) => (*node_a, *node_b),
        _ => panic!("huffman root node should not be a leaf")
    }
}

pub fn decode<X>(input: &[u8], root_node: HuffmanNode<X>) -> Vec<X> where X: Copy {
    let mut br = BitReader::new(input);
    let (rootnode_a, rootnode_b) = get_internals(root_node);

    let mut node = match br.next() {
        Some(false) => &rootnode_a.node_type,
        Some(true) => &rootnode_b.node_type,
        None => panic!("bitreader should not have empty content at beginning")
    };

    let mut output = Vec::new();

    while let Some(bit) = br.next() {
        node = match node {
            NodeType::Leaf(symbol) => {
                output.push(*symbol);
                if bit { &rootnode_b.node_type } else { &rootnode_a.node_type } 
            }
            NodeType::Internal(node_a, node_b) => if bit { &node_b.node_type } else { &node_a.node_type } 
        };
    }

    output
}