use std::{fs::read, collections::HashMap};
use memchr::memmem;

fn read_file_line_by_line(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = read(filepath)?;
    println!("file\n{}", String::from_utf8_lossy(&file[..200]));

    for n in 199..200 {
        let mut rotund : Vec<u8> = Vec::new();

        for le in (1..10).rev() {
            let pat = &file[n-le..n];

            println!("n{} le{} pat:{:?}", n, le, String::from_utf8_lossy(pat));

            let mut counter: HashMap<u8, usize> = HashMap::new();

            for itra in memmem::find_iter(&file[..n], &pat) {
                let next_u8 = file[itra+le];
                println!("{:?} {:?} -> {}", itra, String::from_utf8_lossy(&file[itra..itra+le+1]), next_u8 as char);

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
        println!("rotund{} {}", rotund.len(), String::from_utf8_lossy(&rotund));
    }

    Ok(())
}

fn main() {
    let _ = read_file_line_by_line("../enwik9");
}
