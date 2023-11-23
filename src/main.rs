use std::fs::{read,write};
use std::collections::HashSet;
use std::cmp::Reverse;
use huffman_coding;
use std::io::Write;
use tqdm::tqdm;
use std::env;

fn make_rotund(content: &[u8], markov_order: usize) -> Vec<u8> {
    //let mut rotund_probs = [0; 256];
    let mut rotund_overlap = [0; 256];
    let mut rotund_freq = [0u32; 256];

    let n = content.len();
    if n < markov_order {
        return (0..=255u8).collect()
    }

    let needle = &content[..markov_order];
    let needle_first = *needle.first().unwrap();
    let clen = content.len();

    for a in 0..clen-1 {
        if needle_first != content[a+1] {
            continue;
        } 

        let mut overlap = 1;
        loop {
            let b = overlap+1;
            let c = a+b;
            if (c >= clen) || (needle[overlap] != content[c]) { break; }
            overlap = b;
        }

        let target = content[a] as usize;

        if rotund_overlap[target] > overlap {
            continue
        }
        if rotund_overlap[target] == overlap {
            rotund_freq[target] += 1;
        } else {
            rotund_overlap[target] = overlap;
            rotund_freq[target] = 1;
        }
        //rotund_probs[target] += overlap*overlap*overlap; //cubic
    }

    let mut keys : Vec<u8> = (0..=255u8).collect();
    //keys.sort_by_key(|x| Reverse(&rotund_probs[*x as usize]));
    keys.sort_by_key(|&x| { let xi=x as usize; Reverse((rotund_overlap[xi] << 32) +(rotund_freq[xi] as usize)) });
    keys
    
}

fn generate_probcodes(rfile: &[u8], markov_order: usize) -> Vec<u8> {
    let mut probcodes: Vec<u8> = Vec::new();
    
    for n in tqdm( (1..rfile.len()-1).rev() ) {
        //print!("n{} ", n);
        //let rotund = make_classicrotund(&file[..n], markov_order);
        let rotund = make_rotund(&rfile[n..], markov_order);
        
        //println!("rotund{} {}", rotund.len(), String::from_utf8_lossy(&rotund[..16]));

        let target_u8 = rfile[n-1]; 
        probcodes.push( rotund.iter().position(|&x| x == target_u8).unwrap() as u8 );
    }
    probcodes
}

fn main() {
    const ENWIK9: &str = "../enwik9";
    const UNUSED: &str = "unused.u8";
    let mut args = env::args();
    args.next();

    // next step: make weighted_rotund, which is now linear in length-to-proability, maybe quadratic in len and see what happens
    match args.next().unwrap().as_str() {
        "slice<-enwik" => {
            let maxlen = 400_000;
            let file = read(ENWIK9).unwrap();
            write("enwik.slice", &file[..maxlen]).unwrap();
        },
        "unused<-enwik"=> {
            let file = read(ENWIK9).unwrap();
            let mut symbols: HashSet<u8> = HashSet::from_iter(0..=255u8);
            for ch in tqdm(file.into_iter()) {
                symbols.remove(&ch);
            }
            let contents: Vec<u8> = symbols.into_iter().collect();
            write(UNUSED, contents).unwrap();
        }
        "probcodes<-" => {
            let filename = args.next().unwrap();
            let mut file = read(filename).unwrap();
            file.reverse();
            let probcodes = generate_probcodes(&file, 1000);
            write("probcodes.u8", probcodes).unwrap();
        },
        "entropy<-" => {
            let filename = args.next().unwrap();
            let content = read(filename).unwrap();
            let clen = content.len();
            let mut slots = [0u32; 256];
            for n in content {
                slots[n as usize] += 1;
            }
            println!("slots: {:?} {:?} {:?} {:?} ...", slots[0], slots[1], slots[2], slots[3]);
            let mut entropy = 0.0;
            let su: u32 = slots.iter().sum();
            let sm = su as f64;
            for s in slots.iter().filter(|&x| *x!=0) {
                let p = (*s as f64) / sm;
                entropy -= p * p.log2();
            }

            println!("entropy = {:.4} bits/byte", entropy);
            println!("orig_len = {} bytes", clen);
            println!("entropy*orig_len = {:.1} bytes", (entropy*clen as f64/8.0));
        },
        "huffman<-" => {
            let filename = args.next().unwrap();
            let content = std::fs::read(filename).unwrap();

            let tree = huffman_coding::HuffmanTree::from_data(&content);
            let tree_table = tree.to_table();
            std::fs::write("huffcodes.tree", tree_table).unwrap();

            let mut huffcodes = std::fs::File::create("huffcodes.bin").unwrap(); //Vec::new();
            let mut writer = huffman_coding::HuffmanWriter::new(&mut huffcodes, &tree);
            writer.write(&content).unwrap();
        },
        "rle<-" => {
            let unused_bytes = std::fs::read(UNUSED).unwrap();  
            let filename = args.next().unwrap();  
            let content = std::fs::read(filename).unwrap();
            let mut rle_encoded = Vec::new();

            let mut nulcounter = 0;
            for ch in tqdm(content.into_iter()) {
                if ch==0 {
                    if nulcounter >= unused_bytes.len() {
                        rle_encoded.push(*unused_bytes.last().unwrap());
                        nulcounter = 1;
                    } else if nulcounter != 0 {
                        nulcounter += 1;
                    } else {
                        nulcounter = 1;
                    }
                } else if nulcounter != 0 {
                    nulcounter -= 1;
                    rle_encoded.push(unused_bytes[nulcounter]);
                    nulcounter = 0;
                } else {
                    rle_encoded.push(ch);
                }
            }
            write("rle.u8", rle_encoded).unwrap();
        }
        x => println!("undefined mode: {}", x)
    }
}
