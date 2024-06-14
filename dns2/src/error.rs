use bitarray::buffer;

#[derive(Debug)]
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
}

impl From<buffer::Error> for DnsError {
    fn from(value: buffer::Error) -> Self {
        DnsError::IOError(value)
    }
}
