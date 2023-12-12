use std::cmp::Reverse;
use indicatif::{ProgressBar, ProgressStyle};

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


fn bar(total_size: u64) -> ProgressBar { //from indicatif example "download.rs"
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        //.with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb
}

pub fn encode(reversed: &[u8]) -> Vec<u8> {
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

pub fn decode(probcodes: &[u8]) -> Vec<u8> {
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

#[test]
fn encode_decode() {
    let input = b"This is a simple text for encoding this and that information.".to_vec();
    let mut reversed = input.clone();
    reversed.reverse();
    let encoded = encode(&reversed);
    let mut output = decode(&encoded);
    output.reverse();
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output))
}