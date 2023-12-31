use std::fs::{read,write};

use std::io::{Result, ErrorKind, Error};
use std::env;
use std::path::Path;

mod huffman;
use huffman::{count_freqs, entropy_info, encode, decode, HuffmanNode};

mod prob;
use prob::{encode as prob_encode, decode as prob_decode};

mod prep;
use prep::{prepare, unprepare, unused_symbols};

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
            write(UNUSED_FILE, unused)
        }
        "prepd<-enwik" => {
            let max_len = usize::from_str_radix(&args.next().unwrap(), 10).unwrap();
            let mut input = read(ENWIK9)?;
            input.truncate(max_len);

            let unused = read(UNUSED_FILE)?;
            let mut out = prepare(&input, &unused);
            out.reverse();

            write(prepd_file, &out)
        }
        "probencode<-" => {
            let prepd_filename = args.next().unwrap();

            let prepd = read(prepd_filename).unwrap();
            let probcodes = prob_encode(prepd);
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

            let mut input = read(prepd_file.to_owned()+".d")?;
            input.reverse(); 
            
            let unused = read(UNUSED_FILE)?;
            let out = unprepare(&input, &unused);

            write(filename, out)
        }
        x => Err( Error::new(ErrorKind::NotFound, format!("unknown mode {x}")) )
    }
}
