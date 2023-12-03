use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::Reverse;
use bit_vec::BitVec;
use bitstream::{BitReader, BitWriter};


enum NodeType<X> {
    Internal (Box<HuffmanNode<X>>, Box<HuffmanNode<X>>),
    Leaf(X)
}

pub struct HuffmanNode<X> {
    pub weight: usize,
    node_type: NodeType<X>
}

impl<X> HuffmanNode<X> {
    pub fn new(a: HuffmanNode<X>, b: HuffmanNode<X>) -> Self {
        Self { weight: a.weight + b.weight , node_type: NodeType::Internal(Box::new(a), Box::new(b))}
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


pub fn huffman_code<X>(weights: HashMap<X, usize>) -> HuffmanNode<X> where X: Eq, X: Hash, X: Ord, X: Copy {
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

pub fn gen_dictionary<X>(root_node: HuffmanNode<X>) -> EncodeDict<X> where X: Eq, X: Hash {
    gen_entries(root_node, BitVec::new())
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