use std::fs::{read,write};
use std::collections::HashSet;

use std::io::{Result, ErrorKind, Error};
use std::env;
use std::path::Path;

mod huffman;
use huffman::{count_freqs, entropy_info, encode, decode, HuffmanNode};

mod prob;
use prob::{encode as prob_encode, decode as prob_decode};

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
    let probcodes_file_d = "out/probcodes.u8.d";
    let rle_file = "out/rle.u16";
    let rle_file_d = "out/rle.u16.d";
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
            let xml_end = unused[0]; // used for v1
            let big_char = unused[1]; //used for v1+v2

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
        "rlencode<-" => {
            let filename = args.next().unwrap();
            let mut content = read(filename)?.into_iter();

            let mut rle_pairs: Vec<(u8,u8)> = Vec::new();

            let mut last_nn: u8 = content.next().unwrap();
            let mut nullcounter = 0u8;

            while let Some(ch) = content.next() {
                if ch==0 {
                    if nullcounter == 255u8 {
                        rle_pairs.push( (last_nn, nullcounter) );
                        last_nn = 0u8;
                        nullcounter = 0;
                    } else {
                        nullcounter += 1;
                    }
                } else  {
                    rle_pairs.push( (last_nn, nullcounter) );
                    nullcounter = 0;
                    last_nn = ch;
                }
            }

            rle_pairs.push( (last_nn, nullcounter) ); //flush

            let rle_encoded: Vec<u8> = rle_pairs.into_iter().flat_map(|x| [x.0, x.1]).collect();
            write(rle_file, rle_encoded)
        }
        "entropy<-" => {
            let filename = args.next().unwrap();
            let input = read(filename)?.into_iter();
            entropy_info(count_freqs(input));
            Ok(())
        }
        "entropy16<-" => {
            let filename = args.next().unwrap();
            let input = read_u16(filename)?.into_iter();
            entropy_info(count_freqs(input));
            Ok(())
        }        
        "huffencode8<-" => {
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
        "huffdecode8->" => {
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
            let rle = read(rle_file_d)?;
            let rle_pairs = rle.chunks_exact(2);

            let mut contents = Vec::new();
            for ch in rle_pairs {
                contents.push(ch[0]);
                let nulls = vec![0u8; ch[1] as usize];
                contents.extend(nulls);
            }
            write(filename, contents)
        }
        "probdecode->" => {
            let filename = args.next().unwrap();

            let probcodes = read(probcodes_file_d)?;
            let prepd = prob_decode(&probcodes);
            write(filename, prepd)
        }
        "unprep->" => {
            let filename = args.next().unwrap();

            let mut prepd = read(prepd_file.to_owned()+".d")?;
            prepd.reverse(); 
            
            let unused = read(UNUSED_FILE)?;
            let xml_end = unused[0]; // used for v1
            let big_char = unused[1]; //used for v1+v2

            let mut out: Vec<u8> = Vec::with_capacity(prepd.len());

            let mut xml_tags: Vec<Vec<u8>> = Vec::new();
            let mut n = 0;
            loop {
                let ch = prepd[n];
                match ch {
                    b'<' => {
                        let a = n + 1;
                        let mut b = a + 1;
                        let mut c = 0;
                        while prepd[b] != b'>' {
                            if c==0 && prepd[b] == b' ' { c = b; }
                            b += 1;
                        }
                        if prepd[b-1] != b'/' {
                            if c != 0 { b = c; }
                            xml_tags.push( prepd[a..b].to_vec() );
                        }
                    }
                    _ => ()
                }

                let to_push =
                if ch == big_char {
                    n += 1; prepd[n]-32 //.to_uppercase
                } else if ch == xml_end {
                    out.extend(b"</");
                    out.extend( xml_tags.pop().unwrap() );
                    b'>'
                } else {
                    ch
                }; 
                out.push(to_push);
                n += 1;
                if n>=prepd.len() { break; }
            }
            write(filename, out)
        }
        x => Err( Error::new(ErrorKind::NotFound, format!("unknown mode {x}")) )
    }
}
