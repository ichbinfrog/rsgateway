use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum DNSError {
    MaxRecursionDepth(usize),
}

impl std::fmt::Display for DNSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for DNSError {}
