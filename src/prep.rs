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

enum Tag {
    Opening,
    Closing(usize), //usize are symbol lengths
    Solitary
}

struct PrepState {
    xml_tags: Vec<Vec<u8>>,
    page_title: Vec<u8>
}

impl PrepState {
    pub fn new() -> Self {
        Self { xml_tags: Vec::new(), page_title: Vec::new() }
    }

    fn fetch_xml_tag(&mut self, input: &[u8]) -> Option<Tag> {
        assert_eq!(input[0],b'<');
        let mut b=2;
        let mut c=0;
        while input[b] != b'>' {
            if c==0 && input[b] == b' ' { c = b; }
            b += 1;
            if b>=input.len() { return None }
        }
        if input[b-1] == b'/' {
            Some(Tag::Solitary)
        } else if input[1] == b'/' {
            Some(Tag::Closing(b))
        } else {
            if c != 0 { b = c; }
            self.xml_tags.push( input[1..b].to_vec() );
            if self.xml_tags.last().unwrap() == b"title" {
                b += 1;
                c = b+1;
                while input[c] != b'<' {
                    c += 1;
                }
                self.page_title = input[b..c].to_vec();
            };
            Some(Tag::Opening)
        }
    }

    fn pop_xml_tag(&mut self) -> Vec<u8> {
        let mut out = b"</".to_vec();
        out.extend( self.xml_tags.pop().unwrap() );
        out.push(b'>');
        out       
    }

    fn match_title(&self, input: &[u8], upper: bool) -> usize {
        if self.page_title.is_empty() {
            return 0
        }
        //returns overlap
        if upper && self.page_title[0] != input[0] {
            return 0
        }
        if !upper && self.page_title[0].to_ascii_lowercase() != input[0] {
            return 0
        }
        if self.xml_tags.last().unwrap() == b"title" {
            return 0 //don't replace title with itself..
        }
        for n in 1..self.page_title.len() {
            if self.page_title[n] != input[n] {
                return 0
            }
        }
        self.page_title.len()-1
    }
}

pub fn prepare(input: &[u8], control_chars: &[u8]) -> Vec<u8> {
    let xml_end = control_chars[0]; // used for v1
    let big_char = control_chars[1]; //used for v1+v2
    let title_symbol = control_chars[2]; //used for v3

    let mut out: Vec<u8> = Vec::with_capacity(input.len());

    let mut ps = PrepState::new();
    let mut n = 0;
    loop {
        let ch = input[n];
        let to_push = match ch {
            b'<' => {
                match ps.fetch_xml_tag(&input[n..]) {
                    Some(Tag::Closing(x)) => {
                        n += x;
                        xml_end
                    }
                    _ => ch
                }
            }
            65..=90 => {
                out.push(big_char);
                match ps.match_title(&input[n..], true) {
                    0 => {
                        ch+32 //.to_lowercase
                    }
                    x => {
                        n += x;
                        title_symbol
                    } 
                }
            }
            _ => {
                match ps.match_title(&input[n..], false) {
                    0 => ch,
                    x => {
                        n += x;
                        title_symbol
                    }
                }
            }
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

        if ch == big_char {
            n += 1;
            out.push( input[n]-32 ); //.to_uppercase
        } else if ch == xml_end {
            out.extend( ps.pop_xml_tag() );
        } else {
            out.push(ch);
        }; 
        n += 1;
        if n>=input.len() { break; }
    }
    out
}

#[test]
fn prepare_unprepare() {
    let control_chars = vec![b'~', b'*', b'#'];
    let input = b"<title>Parrot</title><one tag><another tag/>Hi<third tg 2start>this is a test for Basic xml tagging</third>, detecting small parrots and big Parrots, and cApital Letters.</one>".to_vec();
    let prepd = prepare(&input, &control_chars);
    let expected = "<title>*parrot~<one tag><another tag/>*hi<third tg 2start>this is a test for *basic xml tagging~ and c*apital \u{1}letter detection~";
    assert_eq!(expected, String::from_utf8_lossy(&prepd));
    let output = unprepare(&prepd, &control_chars);
    //although assert_eq!(input,output) possible, the following gives better debug info:
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output))
}