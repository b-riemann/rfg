// adapte structure heavily inspired by https://janmr.com/blog/2021/01/rust-and-iterator-adapters/

use std::collections::VecDeque;

pub struct Capsif<I> {
    iter: I,
    cap_symbol: u8,
    store: Option<u8>
}

impl<I> Capsif<I> {
    pub fn new(iter: I, cap_symbol: u8) -> Self {
        Self { iter, cap_symbol, store: None }
    }
    fn pop_stored(&mut self) -> Option<u8> {
        let x = self.store;
        self.store = None;
        x
    }
}

impl<I> Iterator for Capsif<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.store {
            Some(_) => self.pop_stored(),
            None => match self.iter.next() {
                Some(ch) => match ch {
                    65..=90 => { //uppercase
                        self.store = Some(ch+32);
                        Some(self.cap_symbol)
                    }
                    _ => Some(ch)
                }
                None => None
            }
        }
    }
}

pub trait CapsifyIterator: Sized {
    fn capsify(self, cap_symbol: u8) -> Capsif<Self> {
        Capsif::new(self, cap_symbol)
    }
}
impl<I: Iterator> CapsifyIterator for I {}

pub struct UnCapsif<I> {
    iter: I,
    cap_symbol: u8
}

impl<I> Iterator for UnCapsif<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.iter.next() {
            Some(ch) => if ch==self.cap_symbol {
                Some(self.iter.next().unwrap()-32) //make next character uppercase
            } else {
                Some(ch)
            }
            None => None
        }
    }
}

pub trait UnCapsifyIterator: Sized {
    fn uncapsify(self, cap_symbol: u8) -> UnCapsif<Self> {
        UnCapsif{ iter: self, cap_symbol }
    }
}
impl<I: Iterator> UnCapsifyIterator for I {}

pub struct XmlTerminator<I> {
    iter: I,
    term_symbol: u8,
    envs: Vec<Vec<u8>>,
    itercache: VecDeque<u8>
}

impl<I> XmlTerminator<I> {
    pub fn new(iter: I, term_symbol: u8) -> Self {
        Self { iter, term_symbol, envs: Vec::new(), itercache: VecDeque::new() }
    }
}

impl<I> XmlTerminator<I>
where I: Iterator<Item=u8>
{
    fn pop_cache(&mut self) -> Option<u8> {
        if self.itercache.is_empty() {
            self.iter.next()
        } else {
            self.itercache.pop_front()
        }
    }    
}

impl<I> Iterator for XmlTerminator<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let och = self.pop_cache();
        match och {
            Some(b'<') => {
                let mut tag = vec![b'<'];
                loop {
                    match self.pop_cache().expect("stream should not stop in middle of tag") {
                        b'>' => break,
                        x => tag.push(x)
                    }
                }
                if tag[1] == b'/' { // </xml
                    let opener = self.envs.pop().unwrap();
                    assert_eq!(tag[2..], opener[1..tag.len()-1]);
                    Some(self.term_symbol)
                } else if tag.last().unwrap() == &b'/' { // <xml/
                    tag.push(b'>');
                    self.itercache.extend(tag);
                    self.pop_cache()
                } else { // <xml abc> -> xml
                    self.itercache.extend(tag.clone());
                    match tag.clone().into_iter().position(|c| c==b' ') {
                        Some(n) => tag.truncate(n),
                        None => ()
                    };
                    self.itercache.push_back(b'>');
                    tag.push(b'>');
                    self.envs.push(tag);

                    self.pop_cache()
                }
            },
            x => x
        }
    }  
}

pub trait XmltIterator: Sized {
    fn xml_terminate(self, term_symbol: u8) -> XmlTerminator<Self> {
        XmlTerminator::new(self, term_symbol)
    }
}
impl<I: Iterator> XmltIterator for I {}

#[test]
fn capsify_uncapsify() {
    let input = b"this is a Test for Capsif. Are capS escaped correctlY?".to_vec();
    let capsified: Vec<u8> = input.clone().into_iter().capsify(b'^').collect();
    assert_eq!("this is a ^test for ^capsif. ^are cap^s escaped correctl^y?", String::from_utf8_lossy(&capsified));
    let output: Vec<u8> = capsified.into_iter().uncapsify(b'^').collect();
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output));

}

#[test]
fn xmlt_stream() {
    let input = b"this<page> is a Test for <title>XMLT</title>. <one tag><another tag/>It removes<third tg 2start>closing xml environments by an</third>escape character.</one></page>".to_vec().into_iter();
    let output: Vec<u8> = input.xml_terminate(b'+').collect();
    assert_eq!("this<page> is a Test for <title>XMLT+. <one tag><another tag/>It removes<third tg 2start>closing xml environments by an+escape character.++", String::from_utf8_lossy(&output));
}