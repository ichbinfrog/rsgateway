use bitarray::{
    buffer::{self, SizedString},
    serialize::{self, Serialize},
};
use bitarray_derive::Serialize;

// A TCP Header as defined in [RFC-9293](https://datatracker.ietf.org/doc/html/rfc9293#section-3.1)
#[derive(Serialize, PartialEq, Debug)]
pub struct Header<const N: usize> {
    src: u16,
    dst: u16,
    length: u16,
    checksum: u16,
    data: SizedString<N>,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_udp_serialization() {
        
    }
}