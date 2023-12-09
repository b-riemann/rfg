use std::{collections::HashMap, io};
use std::hash::Hash;
use std::cmp::Reverse;
use bit_vec::BitVec;
use bitstream::{BitReader, BitWriter, Padding, LengthPadding};
use std::path::Path;
use std::fs::File;
#[cfg(test)]
use std::io::Cursor;

enum NodeType<X> {
    Internal (Box<HuffmanNode<X>>, Box<HuffmanNode<X>>),
    Leaf(X)
}

pub struct HuffmanNode<X> {
    pub weight: usize,
    node_type: NodeType<X>
}

type EncodeDict<X> = HashMap<X, BitVec>;

fn gen_entries<X>(node: &HuffmanNode<X>, prefix: BitVec) -> EncodeDict<X> where X: Eq, X: Hash, X: Clone {
    let mut dic: EncodeDict<X> = HashMap::new();
    match &node.node_type {
        NodeType::Leaf(sym) => { dic.insert(sym.clone(), prefix); dic }
        NodeType::Internal(node_a, node_b) => {
            let mut prefix_a = prefix.clone();
            prefix_a.push(false);
            dic.extend( gen_entries(&*node_a, prefix_a) );
            let mut prefix_b = prefix.clone();
            prefix_b.push(true);
            dic.extend( gen_entries(&*node_b, prefix_b) );
            dic
        }
    }
}

pub trait SerializedBits {
    fn serialize_to_bits(&self) -> BitVec;
    fn serialize_from_bits(bv: &BitVec) -> Self;
    fn bitlen() -> usize;
}

impl SerializedBits for u8 {
    fn serialize_to_bits(&self) -> BitVec {
        BitVec::from_bytes( &vec![*self] )
    }
    fn serialize_from_bits(bv: &BitVec) -> Self {
        bv.to_bytes()[0]
    }
    fn bitlen() -> usize { 8 }
}

impl SerializedBits for u16 {
    fn serialize_to_bits(&self) -> BitVec {
        BitVec::from_bytes( &self.to_le_bytes().to_vec() )
    }

    fn serialize_from_bits(bv: &BitVec) -> Self {
        let bvb = bv.to_bytes();
        Self::from_le_bytes([bvb[0],bvb[1]])
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

    pub fn encoding_dictionary(&self) -> EncodeDict<X> where X: Eq, X: Hash, X: Clone {
        gen_entries(self, BitVec::new())
    }

    fn to_bitnode(&self) -> BitVec where X: SerializedBits {
        let mut bv = BitVec::new();
        match &self.node_type {
            NodeType::Leaf(symbol) => {
                bv.push(true);
                bv.extend( &symbol.serialize_to_bits() );
            }
            NodeType::Internal(_, _) => {
                bv.push(false);
                bv.extend( self.to_bits() );
            }
        }
        bv
    }

    fn to_bits(&self) -> BitVec where X: SerializedBits {
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

    fn from_bitnode<R,P>(br: &mut BitReader<R, P>) -> Option<(Self, &mut BitReader<R, P>)> where X: SerializedBits, R: std::io::Read, P: Padding  {
        match br.next() {
            Some(true) => {
                let mut bv = BitVec::new();
                for _ in 0..X::bitlen() {
                    match br.next() {
                        Some(bit) => bv.push(bit),
                        None => return None
                    }
                }
                let symbol = X::serialize_from_bits( &bv );
                Some(( Self {weight: 0, node_type: NodeType::Leaf(symbol)}, br ))
            }
            Some(false) => {
                let x = Self::from_bits(br)?;
                Some(x)
            }
            None => None
        }
    }

    fn from_bits<R,P>(br: &mut BitReader<R, P>) -> Option<(Self, &mut BitReader<R, P>)> where X: SerializedBits, R: std::io::Read, P: Padding {
        let (node_a, ba) = Self::from_bitnode(br)?;
        let (node_b, bb) = Self::from_bitnode(ba)?;
        Some(( Self {weight: 0, node_type: NodeType::Internal(Box::new(node_a), Box::new(node_b))}, bb ))
    }

    pub fn to_file<P>(&self, filename: P) -> io::Result<()> where X: SerializedBits, X: std::fmt::Debug, P: AsRef<Path> {
        let mut file = File::create(filename)?;

        // standard bit writer will will the last bits of remaining byte with 0-bits.
        // because of the tree structure with only internal nodes as 0-bits, this is not a problem when reading later.
        let mut bw = BitWriter::new(&mut file);

        let bv = self.to_bits();
        for bit in bv {
            bw.write_bit(bit)?;
        }
        Ok(())
    }

    pub fn from_file<P>(filename: P) -> io::Result<Self> where X: SerializedBits, X: std::fmt::Debug, P: AsRef<Path> {
        let file = File::open(filename)?;
        let mut br = BitReader::new(&file);

        let (hufftree, _) = Self::from_bits(&mut br).unwrap();
        Ok( hufftree )
    }
}

impl<X> std::fmt::Display for HuffmanNode<X> where X: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            NodeType::Leaf(symbol) => write!(f, "_{:?}_", *symbol),
            NodeType::Internal(a, b) => write!(f, "({},{})", *a, *b)
        }
    }
}


pub fn count_freqs<I>(input: I) -> HashMap<I::Item, usize> where I: Iterator, I::Item: Eq, I::Item: Hash {
    let mut counters = HashMap::new();
    for symbol in input {
        let location = counters.entry(symbol).or_insert(0);
        *location += 1;
    }
    counters
}

pub fn encode<I>(input: I, edict: EncodeDict<I::Item>) -> Vec<u8> where I: Iterator, I::Item: Eq, I::Item: PartialEq, I::Item: Hash, I::Item: std::fmt::Debug, I: Clone {
    let mut encoded: Vec<u8> = Vec::new();
    let mut bw = BitWriter::with_padding(&mut encoded, LengthPadding::new());
    for symbol in input.clone() {
        let code = edict.get(&symbol).expect("symbol should be in dictionary");
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

pub fn decode<X>(input: &[u8], root_node: HuffmanNode<X>) -> Vec<X> where X: Copy, X: std::fmt::Debug {
    let mut br = BitReader::with_padding(input, LengthPadding::new());
    let (rootnode_a, rootnode_b) = get_internals(root_node);

    let mut node = match br.next() {
        Some(true) => &rootnode_b.node_type,
        Some(false) => &rootnode_a.node_type,
        None => panic!("bitreader should not have empty content at beginning")
    };

    let mut output = Vec::new();

    loop {
        node = match node {
            NodeType::Leaf(symbol) => {
                output.push(*symbol);
                match br.next() {
                    Some(true) => &rootnode_b.node_type,
                    Some(false) => &rootnode_a.node_type,
                    None => break
                }
            }
            NodeType::Internal(node_a, node_b) => match br.next() {
                Some(true) => &node_b.node_type,
                Some(false) => &node_a.node_type,
                None => break
            }
        };
    }

    output
}

#[test]
fn tree_writevec_readvec() {
    let input: Vec<u16> = vec![3,1,4,1,5,9];
    let freqs = count_freqs(input.into_iter());
    let tree_a = HuffmanNode::from_weights(freqs);

    let bitv = tree_a.to_bits().to_bytes();

    let mut br = BitReader::new(Cursor::new(bitv));
    let x: (HuffmanNode<u16>, &mut BitReader<Cursor<Vec<u8>>, bitstream::NoPadding>) = HuffmanNode::from_bits(&mut br).unwrap();
    let tree_b = x.0;

    let str_a = format!("{}", tree_a);
    let str_b = format!("{}", tree_b);
    assert_eq!(str_a, str_b);
}

#[test]
fn encode_decode() {
    let input_vec: Vec<u16> = vec![3,1,4,1,5,9];
    let input = input_vec.clone().into_iter();
    let freqs = count_freqs(input.clone());
    let tree = HuffmanNode::from_weights(freqs);

    let edict = tree.encoding_dictionary();
    
    let compressed = encode(input, edict);

    let output_vec = decode(&compressed, tree);
    assert_eq!(input_vec, output_vec);
}