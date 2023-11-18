use std::{fs::read, collections::HashMap};
use memchr::memmem;
use std::cmp::min;
use simple_bar::ProgressBar;

fn generate_probcodes(file: &[u8], markov_order: usize) -> Vec<u8> {
    let mut bar = ProgressBar::default(file.len() as u32, 50, true);

    let mut probcodes: Vec<u8> = Vec::new();

    bar.update();
    for n in 1..file.len() {
        //print!("n{} ", n);
        let rotund = make_rotund(&file[..n], markov_order);
        //println!("rotund{} {}", rotund.len(), String::from_utf8_lossy(&rotund[..16]));

        let target_u8 = file[n]; 
        probcodes.push( rotund.iter().position(|&x| x == target_u8).unwrap() as u8 );
        bar.update();
    }
    probcodes
}


fn make_rotund(content: &[u8], markov_order: usize) -> Vec<u8> {
    //rotund is a permutation of the range 0..256. It is ordered so that bytes with highest probability occur first.
    let mut rotund : Vec<u8> = Vec::new();
    let n = content.len();
    let lemax = min(n, markov_order);
    for le in (1..lemax).rev() {
        let pat = &content[n-le..n];

        let mut counter: HashMap<u8, usize> = HashMap::new();

        for itra in memmem::find_iter(&content[..n-1], &pat) {
            let next_u8 = content[itra+le];
            //println!("{:?} {:?} -> {}", itra, String::from_utf8_lossy(&content[itra..itra+le+1]), next_u8 as char);

            if rotund.contains(&next_u8) {
                continue
            }
            let val = counter.entry(next_u8).or_insert(0);
            *val += 1;
        }
        let mut r: Vec<(&u8, &usize)> = counter.iter().collect();
        r.sort_by(|a, b| b.1.cmp(a.1));
        if r.is_empty() {
            continue
        }
        //print!("\nle{} pat:{:?}", le, String::from_utf8_lossy(pat));
        //for etr in &r {
            //print!(" {}: {}, ", *etr.0 as char, etr.1);
        //}

        for ri in r {
            rotund.push(*ri.0);
        }

        if rotund.len() > 256 {
            println!("early stopping!");
            break;
        }
    }
    for xi in 0..=255u8 {
        if !rotund.contains(&xi) {
            rotund.push(xi);
        }
    }
    rotund
}

fn main() {
    let file = read("../enwik9").unwrap();
    //println!("file\n{}", String::from_utf8_lossy(&file[..1000]));
    
    let probcodes = generate_probcodes(&file[..100000], 200);
    //println!("probcode: {:?}", probcodes);
    std::fs::write("probcodes.u8", probcodes).unwrap();
}
