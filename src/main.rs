use std::fs::read;
//use std::collections::HashMap;
//use memchr::memmem;
//use std::cmp::min;
use std::cmp::Reverse;
use huffman_coding;
use std::io::Write;
use tqdm::tqdm;
use std::env;

// fn make_classicrotund(content: &[u8], markov_order: usize) -> Vec<u8> {
//     //rotund is a permutation of the range 0..256. It is ordered so that bytes with highest probability occur first
//     // content: data from which the probabilities are computed
//     // markov_order: largest substring length to be checked

//     let mut rotund : Vec<u8> = Vec::new();
//     let n = content.len();
//     let lemax = min(n, markov_order);
//     for le in (1..lemax).rev() {
//         let pat = &content[n-le..n];

//         let mut counter: HashMap<u8, usize> = HashMap::new();

//         for itra in memmem::find_iter(&content[..n-1], &pat) {
//             let next_u8 = content[itra+le];
//             //println!("{:?} {:?} -> {}", itra, String::from_utf8_lossy(&content[itra..itra+le+1]), next_u8 as char);

//             if rotund.contains(&next_u8) {
//                 continue
//             }
//             let val = counter.entry(next_u8).or_insert(0);
//             *val += 1;
//         }
//         let mut r: Vec<(&u8, &usize)> = counter.iter().collect();
//         r.sort_by(|a, b| b.1.cmp(a.1));
//         if r.is_empty() {
//             continue
//         }
//         //print!("\nle{} pat:{:?}", le, String::from_utf8_lossy(pat));
//         //for etr in &r {
//             //print!(" {}: {}, ", *etr.0 as char, etr.1);
//         //}

//         for ri in r {
//             rotund.push(*ri.0);
//         }

//         if rotund.len() > 256 {
//             println!("early stopping!");
//             break;
//         }
//     }
//     for xi in 0..=255u8 {
//         if !rotund.contains(&xi) {
//             rotund.push(xi);
//         }
//     }
//     rotund
// }

pub fn argsort256(slice: &[usize]) -> Vec<u8> {
    let mut keys : Vec<u8> = (0..=255u8).collect();
    keys.sort_by_key(|x| Reverse(&slice[*x as usize]));
    keys
}


fn make_weightedrotund(content: &[u8], markov_order: usize) -> Vec<u8> {
    //for each occurence, add the next_u8 as a counter
    //then check char before occurance_idx if it fits with the pattern.
    //if so increase the count by 1 
    //if not exit for this occurence_idx and go to next one until markov_order is reached
    let mut rotund_probs = [0; 256];

    let n = content.len();
    if n < markov_order {
        return (0..=255u8).collect()
    }

    let needle = &content[..markov_order];
    let needle_first = *needle.first().unwrap();

    for a in 0..content.len()-markov_order {
        if needle_first != content[a+1] {
            continue;
        } 

        let mut overlap = 1;
        loop {
            let b = overlap+1;
            if needle[overlap] != content[a+b] { break; }
            overlap = b;
            if overlap == markov_order { break; }
        }

        let target = content[a] as usize;
        rotund_probs[target] += overlap*overlap*overlap; //cubic
    }
    argsort256(&rotund_probs)
    
}

fn generate_probcodes(rfile: &[u8], markov_order: usize) -> Vec<u8> {
    let mut probcodes: Vec<u8> = Vec::new();
    
    for n in tqdm( (1..rfile.len()-1).rev() ) {
        //print!("n{} ", n);
        //let rotund = make_classicrotund(&file[..n], markov_order);
        let rotund = make_weightedrotund(&rfile[n..], markov_order);
        
        //println!("rotund{} {}", rotund.len(), String::from_utf8_lossy(&rotund[..16]));

        let target_u8 = rfile[n-1]; 
        probcodes.push( rotund.iter().position(|&x| x == target_u8).unwrap() as u8 );
    }
    probcodes
}

fn main() {
    let mut args = env::args();
    args.next();

    // next step: make weighted_rotund, which is now linear in length-to-proability, maybe quadratic in len and see what happens
    match args.next().unwrap().as_str() {
        "slice<-enwik" => {
            let maxlen = 400_000;
            let file = read("../enwik9").unwrap();
            std::fs::write("enwik.slice", &file[..maxlen]).unwrap();
        },
        "probcodes<-" => {
            let filename = args.next().unwrap();
            let mut file = read(filename).unwrap();
            file.reverse();
            let probcodes = generate_probcodes(&file, 1000);
            std::fs::write("probcodes.u8", probcodes).unwrap();
        },
        "entropy<-" => {
            let filename = args.next().unwrap();
            let probcodes = read(filename).unwrap();
            let mut slots = [0u32; 256];
            for n in probcodes {
                slots[n as usize] += 1;
            }
            //println!("{:?}", slots);
            let mut entropy = 0.0;
            let su: u32 = slots.iter().sum();
            let sm = su as f64;
            for s in slots.iter().filter(|&x| *x!=0) {
                let p = (*s as f64) / sm;
                entropy -= p * p.log2();
            }
            println!("entropy = {:.4} bits/byte", entropy);
        },
        "huffman<-" => {
            let filename = args.next().unwrap();
            let probcodes = std::fs::read(filename).unwrap();

            let tree = huffman_coding::HuffmanTree::from_data(&probcodes);
            let tree_table = tree.to_table();
            std::fs::write("huffcodes.tree", tree_table).unwrap();

            let mut huffcodes = std::fs::File::create("huffcodes.bin").unwrap(); //Vec::new();
            let mut writer = huffman_coding::HuffmanWriter::new(&mut huffcodes, &tree);
            writer.write(&probcodes).unwrap();
        },
        x => println!("undefined mode: {}", x)
    }
}
