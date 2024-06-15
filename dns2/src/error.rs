use bitarray::buffer;

#[derive(Debug, PartialEq)]
pub enum DnsError {
    TooManyJumps {
        max: usize,
    },
    TooLarge {
        max: usize,
        size: usize,
        subject: &'static str,
    },
    IOError(buffer::Error),
    Unimplemented {
        sub: &'static str,
        reason: &'static str,
    },
}

impl From<buffer::Error> for DnsError {
    fn from(value: buffer::Error) -> Self {
        DnsError::IOError(value)
    }
}
