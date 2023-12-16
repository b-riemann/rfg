use std::{cmp::Reverse, collections::HashMap};
use indicatif::{ProgressBar, ProgressStyle};

struct RotundHelper {
    reversed: Vec<u8>,
    cache: HashMap<u8,Vec<usize>>
}

impl RotundHelper {
    pub fn new(reversed: Vec<u8>) -> Self {
        Self { reversed, cache: HashMap::new() }
    }

    fn make_rotund(&self, n: usize) -> Vec<u8> {
        let content = &self.reversed[n..];

        let mut rotund_overlap = [0; 256];
        let mut rotund_freq = [0u32; 256];

        let first = content[0];
        let clen = content.len();

        let vn: Vec<usize> = Vec::new();
        let entries = match self.cache.get(&first) {
            Some(entr) => entr,
            None => &vn
        };

        for x in entries {
            let a = x-n-1;
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
        keys.sort_by_key(|&x| { let xi=x as usize; Reverse((rotund_overlap[xi] << 32) +(rotund_freq[xi] as usize)) });
        keys
    }

    fn add_to_cache(&mut self, n: usize) {
        let key = self.reversed[n];
        let val_vec = self.cache.entry(key).or_insert(Vec::new());
        val_vec.push(n)
    }
}

fn bar(total_size: u64) -> ProgressBar { //from indicatif example "download.rs"
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        //.with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb
}

pub fn encode(reversed: Vec<u8>) -> Vec<u8> {
    let nm_end = reversed.len();
    let mut probcodes = vec![0u8; nm_end];
    
    let mut m = 0;
    let mut n = nm_end - 1;
    probcodes[m] = reversed[n];

    let mut helper = RotundHelper::new(reversed);

    let pb = bar(nm_end as u64);
    loop {
        let rotund = helper.make_rotund(n);
        helper.add_to_cache(n);
        n -= 1;
        let target_u8 = helper.reversed[n];
        m += 1;
        probcodes[m] = rotund.iter().position(|&x| x == target_u8).unwrap() as u8;
        if m % 32 == 0 { pb.set_position(m as u64); }
        if n == 0 { pb.set_position(nm_end as u64); break }
    }
    probcodes
}

pub fn decode(probcodes: &[u8]) -> Vec<u8> {
    let nm_end = probcodes.len();
    let mut helper = RotundHelper::new(vec![0u8; nm_end]);

    let mut m = 0;
    let mut n = nm_end - 1;
    helper.reversed[n] = probcodes[m];

    let pb = bar(nm_end as u64);
    loop {
        let rotund = helper.make_rotund(n);
        helper.add_to_cache(n);
        m += 1;
        let ch = rotund[probcodes[m] as usize];
        n -= 1;
        helper.reversed[n] = ch;
        if m % 32 == 0 { pb.set_position(m as u64); }
        if n == 0 { pb.set_position(nm_end as u64); break }
    }
    helper.reversed
}

#[test]
pub fn encode_decode() {
    let input = b"This is a simple text for encoding this and that information.".to_vec();
    let mut reversed = input.clone();
    reversed.reverse();
    let encoded = encode(reversed);
    let expected = "This i\0\0b sinple text!ior iocoeiog \u{5}h\0\0\0\u{1}ne!\u{1}\u{1}bt\u{1}\u{4}\u{2}g\0\0mb\u{2}ipo2";
    assert_eq!(expected, String::from_utf8(encoded.clone()).unwrap());
    let mut output = decode(&encoded);
    output.reverse();
    assert_eq!(String::from_utf8(input).unwrap(), String::from_utf8(output).unwrap())
}