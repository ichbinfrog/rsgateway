pub mod tun;
use std::{
    net::{SocketAddrV4, ToSocketAddrs},
    os::fd::RawFd,
};

use bitarray::{
    buffer::{self, SizedString},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

// An UDP Packet as defined in [RFC-768](https://datatracker.ietf.org/doc/html/rfc768)
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Packet<const N: usize> {
    src: u16,
    dst: u16,
    length: u16,
    checksum: u16,

    #[rustfmt::skip]
    data: SizedString::<N>,
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;

    use buffer::Buffer;

    use super::*;

    #[test]
    fn test_udp_serialization() {
        let header = Packet::<2> {
            src: 1,
            dst: 2,
            checksum: 3,
            length: 32,
            data: SizedString("holla".to_string()),
        };
        let mut buf = Buffer::new(252);
        let n = header.serialize(&mut buf).unwrap();
        buf.reset();

        let (res, m) = Packet::<2>::deserialize(&mut buf).unwrap();
        assert_eq!(n, m);
        assert_eq!(header, res);
    }
}