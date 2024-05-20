use std::error::Error;

#[derive(Debug)]
pub enum LookupError {
    MaxRecursionDepth(usize),
    IOError(std::io::Error),
    PacketError(PacketError),
}

impl std::fmt::Display for LookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for LookupError {}
impl From<std::io::Error> for LookupError {
    fn from(src: std::io::Error) -> Self {
        Self::IOError(src)
    }
}
impl From<PacketError> for LookupError {
    fn from(src: PacketError) -> Self {
        Self::PacketError(src)
    }
}

#[derive(Debug)]
pub enum PacketError {
    ContentTooLarge { max_size: usize },
    LabelTooLarge { max_size: usize },
    OutOfBound { index: usize },
    InvalidBound { start: usize, end: usize },
    TooManyJumps,
    NotImplemented { reason: String },
    IOError(std::io::Error),
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for PacketError {}
impl From<std::io::Error> for PacketError {
    fn from(src: std::io::Error) -> Self {
        Self::IOError(src)
    }
}
