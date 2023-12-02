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
    let symbols: Vec<usize> = (0..weights.len()).filter(|&w| w!=0).collect();

    let mut nodes: Vec<HuffmanNode> = symbols.into_iter().map(|sym| HuffmanNode { weight: weights[sym], node_type: NodeType::Leaf(sym) }).collect();
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

type HuffDict = HashMap<usize, BitVec>;

fn gen_entries(node: HuffmanNode, prefix: BitVec) -> HuffDict {
    let mut dic: HuffDict = HashMap::new();
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

pub fn gen_dictionary(root_node: HuffmanNode) -> HuffDict {
    gen_entries(root_node, BitVec::new())
}

pub fn encode(input: &[usize], dic: HuffDict) -> Vec<u8> {
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