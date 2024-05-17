// https://datatracker.ietf.org/doc/html/rfc1951#section-Abstract
#[derive(Debug)]
pub enum Method {
    NoCompression { length: usize },
}

#[derive(Debug)]
pub struct Block {
    last: bool,
    method: Method,
}
