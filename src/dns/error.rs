use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum LookupError {
    MaxRecursionDepth(usize),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for LookupError {}

#[derive(Debug)]
pub enum PacketError {
    ContentTooLarge { max_size: usize },
    LabelTooLarge { max_size: usize },
    OutOfBound { index: usize },
    InvalidBound { start: usize, end: usize },
    TooManyJumps,
    NotImplemented { reason: String },
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for PacketError {}
