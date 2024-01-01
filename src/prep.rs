use std::collections::HashSet;

fn order_symbols(symbols: HashSet<u8>) -> Vec<u8> {
    let mut s : Vec<u8> = symbols.into_iter().collect();
    s.sort_by(|a,b| a.cmp(b));
    s
}

pub fn unused_symbols(content: &[u8]) -> Vec<u8> {
    let mut symbols: HashSet<u8> = HashSet::from_iter(0..=255u8);
    for ch in content {
        symbols.remove(&ch);
    }
    order_symbols(symbols)
}

struct PrepState {
    xml_tags: Vec<Vec<u8>>
}

impl PrepState {
    pub fn new() -> Self {
        Self { xml_tags: Vec::new() }
    }

    fn fetch_xml_tag(&mut self, input: &[u8]) {
        assert_eq!(input[0],b'<');
        let mut b=2;
        let mut c=0;
        while input[b] != b'>' {
            if c==0 && input[b] == b' ' { c = b; }
            b += 1;
            if b>=input.len() { break; }
        }
        if input[b-1] == b'/' { return }
        if c != 0 { b = c; }
        self.xml_tags.push( input[1..b].to_vec() )
    }

    fn pop_xml_tag(&mut self) -> Vec<u8> {
        self.xml_tags.pop().unwrap()
    }
}

pub fn prepare(input: &[u8], control_chars: &[u8]) -> Vec<u8> {
    let xml_end = control_chars[0]; // used for v1
    let big_char = control_chars[1]; //used for v1+v2

    let mut out: Vec<u8> = Vec::with_capacity(input.len());

    let mut ps = PrepState::new();
    let mut n = 0;
    loop {
        let ch = input[n];
        let to_push = match ch {
            b'<' => {
                ps.fetch_xml_tag(&input[n..]);
                if input[n+1] != b'/' {
                    ch
                } else {
                    n += 2;
                    while input[n] != b'>' && n<input.len() {
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
        if n>=input.len() { break; }
    }
    out
}



pub fn unprepare(input: &[u8], control_chars: &[u8]) -> Vec<u8> {
    let xml_end = control_chars[0]; // used for v1
    let big_char = control_chars[1]; //used for v1+v2

    let mut out: Vec<u8> = Vec::with_capacity(input.len());

    let mut ps = PrepState::new();
    let mut n = 0;
    loop {
        let ch = input[n];
        match ch {
            b'<' => {
                ps.fetch_xml_tag(&input[n..]);
            }
            _ => ()
        }

        let to_push =
        if ch == big_char {
            n += 1; input[n]-32 //.to_uppercase
        } else if ch == xml_end {
            out.extend(b"</");
            out.extend( ps.pop_xml_tag() );
            b'>'
        } else {
            ch
        }; 
        out.push(to_push);
        n += 1;
        if n>=input.len() { break; }
    }
    out
}

#[test]
fn prepare_unprepare() {
    let control_chars = vec![b'~', 1u8];
    let input = b"<one tag><another tag/>Hi<third tg 2start>this is a test for Basic xml tagging</third> and cApital Letter detection</one>".to_vec();
    let prepd = prepare(&input, &control_chars);
    let output = unprepare(&prepd, &control_chars);
    //although assert_eq!(input,output) possible, the following gives better debug info:
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output))
}