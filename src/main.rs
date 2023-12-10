use std::fs::{read,write};
use std::collections::HashSet;

use std::cmp::Reverse;
use std::io::{Result, ErrorKind, Error};
use std::env;
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};

mod huffman;
use huffman::{count_freqs, entropy_info, encode, decode, HuffmanNode};

fn bar(total_size: u64) -> ProgressBar { //from indicatif example "download.rs"
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        //.with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb
}

fn make_rotund(content: &[u8]) -> Vec<u8> {
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

fn read_u16<P>(path: P) -> Result<Vec<u16>> where P: AsRef<Path> {
    let contents = read(path)?;
    let contents_u16: Vec<u16> = contents.chunks_exact(2).map(|bytes| u16::from_le_bytes([bytes[0],bytes[1]])).collect();
    Ok(contents_u16)
}

fn write_u16<P>(path: P, contents: Vec<u16>) -> Result<()> where P: AsRef<Path> {
    let contents_u8: Vec<u8> = contents.into_iter().flat_map(|b| b.to_le_bytes()).collect();
    write(path, contents_u8)
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
            let mut file = read(ENWIK9)?;
            file.truncate(max_len);

            let unused = read(UNUSED_FILE)?;

            let xml_end = unused[0];
            let big_char = unused[1];

            let mut out: Vec<u8> = Vec::with_capacity(max_len);
            let mut n = 0;
            loop {
                let ch = file[n];
                let to_push = match ch {
                    b'<' => {
                        if file[n+1] != b'/' {
                            ch
                        } else {
                            n += 2;
                            while file[n] != b'>' {
                                n += 1;
                            }
                            xml_end
                        }
                    }
                    65..=90 => {
                        out.push(big_char);
                        ch+32 //.to_lowercase
                    }
                    _ => ch
                };
                out.push( to_push );

                n += 1;
                if n>=max_len { break; }
            }

            out.reverse();
            write(prepd_file, &out)
        }
        "probencode<-" => {
            let prepd_filename = args.next().unwrap();

            let prepd = read(prepd_filename).unwrap();
            let probcodes = prob_encode(&prepd);
            write(probcodes_file, probcodes)
        }
        "defunct:rlencode<-" => {
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
            let input = read(filename)?;

            let freqs = count_freqs(input.into_iter());
            entropy_info(freqs);
            Ok(())
        }
        "huffencode<-" => {
            let filename = args.next().unwrap();
            let contents = read(filename)?;
            let input = contents.into_iter();

            let freqs = count_freqs(input.clone());
            let tree = HuffmanNode::from_weights(freqs);
            println!("{}", tree);
            tree.to_file(hufftree_file)?;

            let out = encode(input, &tree);
            write(huffbin_file, out)
        }
        "huffencode16<-" => {
            let filename = args.next().unwrap();
            let contents = read_u16(filename)?;
            let input = contents.into_iter();

            let freqs = count_freqs(input.clone());
            let tree = HuffmanNode::from_weights(freqs);
            println!("{}", tree);
            tree.to_file(hufftree_file)?;

            let out = encode(input, &tree);
            write(huffbin_file, out)
        }
        "huffdecode->" => {
            let filename = args.next().unwrap();

            let tree: HuffmanNode<u8> = HuffmanNode::from_file(hufftree_file)?;
            println!("{}", tree);
            
            let input = read(huffbin_file)?;
            let output = decode(&input, tree);
            write(filename, output)
        }
        "huffdecode16->" => {
            let filename = args.next().unwrap();

            let tree: HuffmanNode<u16> = HuffmanNode::from_file(hufftree_file)?;
            println!("{}", tree);
            
            let input = read(huffbin_file)?;
            let output = decode(&input, tree);
            write_u16(filename, output)
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
