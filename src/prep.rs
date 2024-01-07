// adapte structure heavily inspired by https://janmr.com/blog/2021/01/rust-and-iterator-adapters/

use std::collections::VecDeque;

pub struct Capsif<I> {
    iter: I,
    cap_symbol: u8,
    store: Option<u8>
}

impl<I> Iterator for Capsif<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        match self.store {
            Some(x) => { self.store = None; Some(x) }
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
        Capsif { iter: self, cap_symbol, store: None }
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

struct EnvIterator<I> {
    iter: I,
    envs: Vec<Vec<u8>>,
    itercache: VecDeque<u8>
}

impl<I> EnvIterator<I> {
    fn new(iter: I) -> Self {
        Self{ iter, envs: Vec::new(), itercache: VecDeque::new() }
    }
}

impl<I> EnvIterator<I>
where I: Iterator<Item=u8>
{
    fn pop(&mut self) -> Option<u8> {
        if self.itercache.is_empty() {
            self.iter.next()
        } else {
            self.itercache.pop_front()
        }
    }

    fn extend_with_end(&mut self, tag: &[u8]) {
        self.itercache.extend(tag);
        self.itercache.push_back(b'>')
    }

    fn push_env_with_end(&mut self, mut tag: Vec<u8>) {
        tag.push(b'>');
        self.envs.push(tag)
    }

    fn pop_env(&mut self) -> Vec<u8> {
        self.envs.pop().unwrap()
    }

    fn get_next_tag(&mut self) -> Vec<u8> {
        let mut tag = vec![b'<'];
        loop {
            match self.pop().expect("stream should not stop in middle of tag") {
                b'>' => break,
                x => tag.push(x)
            }
        }
        tag
    }

    fn singular_or_opener(&mut self, mut tag: Vec<u8>) -> Option<u8> {
        if tag.last().unwrap() == &b'/' { // <xml/
            self.extend_with_end(&tag);
        } else { // <xml abc> -> xml
            self.extend_with_end(&tag);
            match tag.clone().into_iter().position(|c| c==b' ') {
                Some(n) => tag.truncate(n),
                None => ()
            };
            self.push_env_with_end(tag);
        }
        self.pop()
    }
}

pub struct XmlTerminator<I> {
    envi: EnvIterator<I>,
    term_symbol: u8
}

impl<I> Iterator for XmlTerminator<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let och = self.envi.pop();
        match och {
            Some(b'<') => {
                let tag = self.envi.get_next_tag();
                if tag[1] == b'/' { // </xml
                    let opener = self.envi.pop_env();
                    assert_eq!(tag[2..], opener[1..tag.len()-1]);
                    Some(self.term_symbol)
                } else {
                    self.envi.singular_or_opener(tag)
                }
            },
            x => x
        }
    }  
}

pub trait XmltIterator: Sized {
    fn xml_terminate(self, term_symbol: u8) -> XmlTerminator<Self> {
        XmlTerminator { envi: EnvIterator::new(self), term_symbol }
    }
}
impl<I: Iterator> XmltIterator for I {}

pub struct XmlUnterminator<I> {
    envi: EnvIterator<I>,
    term_symbol: u8
}


impl<I> Iterator for XmlUnterminator<I>
where I: Iterator<Item=u8>
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let och = self.envi.pop();
        match och {
            Some(b'<') => {
                let tag = self.envi.get_next_tag();
                self.envi.singular_or_opener(tag)
            }
            Some(x) => if x == self.term_symbol {
                let opener = self.envi.pop_env();
                let mut tag = b"</".to_vec();
                tag.extend(&opener[1..opener.len()-1]);
                self.envi.extend_with_end(&tag);
                self.envi.pop()
            } else {
                Some(x)
            }
            None => None
        }
    }  
}

pub trait XmlutIterator: Sized {
    fn xml_unterminate(self, term_symbol: u8) -> XmlUnterminator<Self> {
        XmlUnterminator { envi: EnvIterator::new(self), term_symbol }
    }
}
impl<I: Iterator> XmlutIterator for I {}

#[test]
fn capsify_uncapsify() {
    let input = b"this is a Test for Capsif. Are capS escaped correctlY?".to_vec();
    let capsified: Vec<u8> = input.clone().into_iter().capsify(b'^').collect();
    assert_eq!("this is a ^test for ^capsif. ^are cap^s escaped correctl^y?", String::from_utf8_lossy(&capsified));
    let output: Vec<u8> = capsified.into_iter().uncapsify(b'^').collect();
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output));
}

#[test]
fn xml_terminate_unterminate() {
    let input = b"this<page> is a Test for <title>XMLT</title>. <one tag><another tag/>It removes<third tg 2start>closing xml environments by an</third>escape character.</one></page>".to_vec();
    let xml_terminated: Vec<u8> = input.clone().into_iter().xml_terminate(b'+').collect();
    assert_eq!("this<page> is a Test for <title>XMLT+. <one tag><another tag/>It removes<third tg 2start>closing xml environments by an+escape character.++", String::from_utf8_lossy(&xml_terminated));
    let output: Vec<u8> = xml_terminated.into_iter().xml_unterminate(b'+').collect();
    assert_eq!(String::from_utf8_lossy(&input), String::from_utf8_lossy(&output));
}