use bitstream::BitReader;
use std::collections::HashMap;
use std::cmp::Reverse;
use bit_vec::BitVec;
use bitstream::BitWriter;

enum NodeType {
    Internal (Box<HuffmanNode>, Box<HuffmanNode>),
    Leaf(usize)
}

pub struct HuffmanNode {
    pub weight: usize,
    node_type: NodeType
}

impl HuffmanNode {
    pub fn new(a: HuffmanNode, b: HuffmanNode) -> Self {
        Self { weight: a.weight + b.weight , node_type: NodeType::Internal(Box::new(a), Box::new(b))}
    }
}

pub fn huffman_code(weights: &[usize]) -> HuffmanNode {
    let occuring_symbols = (0..weights.len()).filter(|&w| w!=0);

    let mut nodes: Vec<HuffmanNode> = occuring_symbols.map(|sym| HuffmanNode { weight: weights[sym], node_type: NodeType::Leaf(sym) }).collect();
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

type EncodeDict = HashMap<usize, BitVec>;

fn gen_entries(node: HuffmanNode, prefix: BitVec) -> EncodeDict {
    let mut dic: EncodeDict = HashMap::new();
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

pub fn gen_dictionary(root_node: HuffmanNode) -> EncodeDict {
    gen_entries(root_node, BitVec::new())
}

pub fn encode(input: &[usize], dic: EncodeDict) -> Vec<u8> {
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

fn get_internals(root_node: HuffmanNode) -> (HuffmanNode, HuffmanNode) {
    match root_node.node_type {
        NodeType::Internal(node_a, node_b) => (*node_a, *node_b),
        _ => panic!("huffman root node should not be a leaf")
    }
}

pub fn decode(input: &[u8], root_node: HuffmanNode) -> Vec<usize> {
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