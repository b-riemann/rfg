use std::fs::{read,write};
use std::collections::HashSet;

use std::cmp::Reverse;
use std::io::{Result, ErrorKind, Error};
use std::env;

use indicatif::{ProgressBar, ProgressStyle};

mod huffman;
use huffman::{count_freqs, encode, decode, HuffmanNode};

fn bar(total_size: u64) -> ProgressBar { //from indicatif example "download.rs"
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        //.with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb
}

fn make_rotund(content: &[u8]) -> Vec<u8> {
    //let mut rotund_probs = [0; 256];
    let mut rotund_overlap = [0; 256];
    let mut rotund_freq = [0u32; 256];

    let first = content[0];
    let clen = content.len();

    for a in 0..clen-1 {
        if first != content[a+1] {
            continue;
        } 

        let mut overlap = 1;
        loop {
            let b = overlap+1;
            let c = a+b;
            if (c >= clen) || (content[overlap] != content[c]) { break; }
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

fn prob_encode(reversed: &[u8]) -> Vec<u8> {
    let nm_end = reversed.len();
    let mut probcodes = vec![0u8; nm_end];
    
    let mut m = 0;
    let mut n = nm_end - 1;
    probcodes[m] = reversed[n];

    let pb = bar(nm_end as u64);
    loop {
        let rotund = make_rotund(&reversed[n..]);
        n -= 1;
        let target_u8 = reversed[n];
        m += 1;
        probcodes[m] = rotund.iter().position(|&x| x == target_u8).unwrap() as u8;
        if m % 32 == 0 { pb.set_position(m as u64); }
        if n == 0 { pb.set_position(nm_end as u64); break }
    }
    probcodes
}

fn prob_decode(probcodes: &[u8]) -> Vec<u8> {
    let nm_end = probcodes.len();
    let mut reversed = vec![0u8; nm_end];

    let mut m = 0;
    let mut n = nm_end - 1;
    reversed[n] = probcodes[m];

    let pb = bar(nm_end as u64);
    loop {
        let rotund = make_rotund(&reversed[n..]);
        m += 1;
        let ch = rotund[probcodes[m] as usize];
        n -= 1;
        reversed[n] = ch;
        if m % 32 == 0 { pb.set_position(m as u64); }
        if n == 0 { pb.set_position(nm_end as u64); break }
    }
    reversed
}

fn order_symbols(symbols: HashSet<u8>) -> Vec<u8> {
    let mut s : Vec<u8> = symbols.into_iter().collect();
    s.sort_by(|a,b| a.cmp(b));
    s
}

fn unused_symbols(content: &[u8]) -> Vec<u8> {
    let mut symbols: HashSet<u8> = HashSet::from_iter(0..=255u8);
    for ch in content {
        symbols.remove(&ch);
    }
    order_symbols(symbols)
}

fn used_from(unused_symbols: &[u8]) -> Vec<u8> {
    let mut used: HashSet<u8> = HashSet::from_iter(0..=255u8);
    for ch in unused_symbols {
        used.remove(&ch);
    }
    order_symbols(used)
}


// fn read_u16(filename: String) -> Result<Vec<u16>> {
//     let contents = read(filename)?;
//     let contents_u16: Vec<u16> = contents.chunks_exact(2).map(|bytes| u16::from_le_bytes([bytes[0],bytes[1]])).collect();
//     Ok(contents_u16)
// }

fn dummy_tree() -> HuffmanNode<u16> {
    let contents: Vec<u16> = vec![1,1,2,2,2,6,6,4,3,3,3,3,3,5,7,7,7,7,8,8,9,9];
    let freqs = count_freqs(contents.into_iter());
    HuffmanNode::from_weights(freqs)
}

fn main() -> Result<()> {
    const ENWIK9: &str = "../enwik9";
    const UNUSED_FILE: &str = "unused.u8";
    const USED_FILE: &str = "used.u8";
    let prepd_file = "out/enwik.prepd";
    let probcodes_file = "out/probcodes.u8";
    let probcodes_file_d = "out/probcodes.u8.decompressed";
    let rle_file = "out/rle.u8";
    let rle_file_d = "out/rle.u8.decompressed";
    let hufftree_file = "out/huffcodes.tree";
    let huffbin_file = "out/huffcodes.bin";

    let mut args = env::args();
    args.next();

    // the order of possible modes corresponds to one compression->decompression cycle (with entropy diagnosis in between)
    match args.next().unwrap().as_str() {
        "unused<-enwik"=> {
            let file = read(ENWIK9)?;
            let unused = unused_symbols(&file);
            let used = used_from(&unused);
            write(UNUSED_FILE, unused)?;
            write(USED_FILE, used)
        }
        "prepd<-enwik" => {
            let max_len = usize::from_str_radix(&args.next().unwrap(), 10).unwrap();

            let used = read(USED_FILE)?;
            let mut usedix = [0u8; 256];
            for (n, ch) in used.into_iter().enumerate() {
                usedix[ch as usize] = n as u8;
            }

            let mut file = read(ENWIK9)?;
            file.truncate(max_len);
            file.reverse();
            let prepd : Vec<u8> = file.into_iter().map(|x| usedix[x as usize]).collect();
            write(prepd_file, &prepd)
        }
        "probencode<-" => {
            let prepd_filename = args.next().unwrap();

            let prepd = read(prepd_filename).unwrap();
            let probcodes = prob_encode(&prepd);
            write(probcodes_file, probcodes)
        }
        "rlencode<-" => {
            let filename = args.next().unwrap();

            //available unused bytecode range for RLE encosing is offset..=255u8
            //the null values to run-length encode range from 1..maxcount
            let offset = read(USED_FILE)?.len() as u8 - 1;
            let maxcount = 255u8-offset-1; 
  
            let content = read(filename)?;
            let mut rle_encoded = Vec::new();

            let mut nullcounter = 0u8;
            for ch in content {
                if ch==0 {
                    if nullcounter > maxcount {
                        rle_encoded.push(offset+nullcounter);
                        nullcounter = 1;
                    } else {
                        nullcounter += 1;
                    }
                } else if nullcounter != 0 {
                    rle_encoded.push(offset+nullcounter);
                    nullcounter = 0;
                    rle_encoded.push(ch)
                } else {
                    rle_encoded.push(ch);
                }
            }

            if nullcounter != 0 { // flush the nulls
                rle_encoded.push(offset+nullcounter);
            }
            write(rle_file, rle_encoded)
        }
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
            Ok(())
        }
        "huffencode<-" => {
            let filename = args.next().unwrap();
            let contents = read(filename)?;
            let input = contents.into_iter();

            let freqs = count_freqs(input.clone());
            let tree = HuffmanNode::from_weights(freqs);
            tree.to_file(hufftree_file)?;

            let edict = tree.encoding_dictionary();
            let out = encode(input, edict);
            write(huffbin_file, out)
        }
        "test:huffencode16bit" => {
            let tree = dummy_tree();
            tree.to_file("tree.test")?;
            let edict = tree.encoding_dictionary();
            println!("{:?}", edict);
            let message = vec![3,1,4,1,5,9].into_iter();
            let encoded = encode(message, edict);
            print!("encoded stream:");
            let _: Vec<_> = encoded.into_iter().map(|num| print!(" {:08b}({})", num, num)).collect();
            Ok(())
        }
        "huffdecode->" => {
            let filename = args.next().unwrap();

            let tree: HuffmanNode<u8> = HuffmanNode::from_file(hufftree_file)?;

            let input = read(huffbin_file)?;
            let output = decode(&input, tree);
            write(filename, output)
        }
        "test:huffdecode16bit" => {
            //let tree = dummy_tree();
            let tree: HuffmanNode<u16> = HuffmanNode::from_file("tree.test")?;

            let encoded = vec![151,87,60];
            let decoded = decode(&encoded, tree);
            println!("decoded {:?}", decoded);
            Ok(())
        }
        "rldecode->" => {
            let filename = args.next().unwrap();

            let offset = read(USED_FILE)?.len() as u8 - 1;
            let rle = read(rle_file_d)?;
            let mut contents = Vec::new();
            for ch in rle {
                if ch > offset {
                    let mut nulls = vec![0u8; (ch-offset) as usize];
                    contents.append(&mut nulls);
                } else {
                    contents.push(ch);
                }
            }
            write(filename, contents)
        }
        "probdecode->" => {
            let filename = args.next().unwrap();

            let probcodes = read(probcodes_file_d)?;
            let prepd = prob_decode(&probcodes);
            write(filename, prepd)
        }
        x => Err( Error::new(ErrorKind::NotFound, format!("unknown mode {x}")) )
    }
}
