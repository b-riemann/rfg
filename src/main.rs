use std::{fs::read, collections::HashMap};
use memchr::memmem;
use std::cmp::min;

fn read_file_line_by_line(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = read(filepath)?;
    println!("file\n{}", String::from_utf8_lossy(&file[..200]));

    for n in 199..200 {
        let rotund = make_rotund(&file[..n]);
        println!("{} rotund{} {}", n, rotund.len(), String::from_utf8_lossy(&rotund));
    }
    Ok(())
}


fn make_rotund(content: &[u8]) -> Vec<u8> {
    //rotund is a permutation of the range 0..256. It is ordered so that bytes with highest probability occur first.
    let mut rotund : Vec<u8> = Vec::new();
    let n = content.len();
    let lemax = min(n, 10);
    for le in (1..lemax).rev() {
        let pat = &content[n-le..n];

        println!("le{} pat:{:?}", le, String::from_utf8_lossy(pat));

        let mut counter: HashMap<u8, usize> = HashMap::new();

        for itra in memmem::find_iter(&content[..n-1], &pat) {
            let next_u8 = content[itra+le];
            println!("{:?} {:?} -> {}", itra, String::from_utf8_lossy(&content[itra..itra+le+1]), next_u8 as char);

            if rotund.contains(&next_u8) {
                continue
            }
            let val = counter.entry(next_u8).or_insert(0);
            *val += 1;
        }
        let mut r: Vec<(&u8, &usize)> = counter.iter().collect();
        r.sort_by(|a, b| b.1.cmp(a.1));
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
    let _ = read_file_line_by_line("../enwik9");
}
