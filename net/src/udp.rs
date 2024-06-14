use bitarray::{
    buffer::{self, SizedString},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

// An UDP Packet as defined in [RFC-768](https://datatracker.ietf.org/doc/html/rfc768)
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Header {
    src: u16,
    dst: u16,
    length: u16,
    checksum: u16,
}

#[derive(Debug)]
pub struct Datagram {
    header: Header,
    pub data: Vec<u8>,
}

impl Deserialize for Datagram {
    type Err = buffer::Error;
    fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        let (header, header_l) = Header::deserialize(buf)?;
        let data = buf.read_exact_n(header.length as usize - (header_l / buffer::BYTE_SIZE) - 1)?;
        let data_l = data.len();
        Ok((Self { header, data }, header_l + data_l))
    }
}

#[cfg(test)]
pub mod tests {

    use buffer::Buffer;

    use crate::ip;

    use super::*;

    #[test]
    fn test_udp_deserialization() {
        let raw = vec![
            0x0, 0x0, // tun_flags
            0x8, 0x0,  // tun_proto
            0x45, // version | ihl
            0x0,  // tos
            0x0, 0x22, // length
            0x36, 0xcb, // ident
            0x40, 0x0,  // flags[3] || offset[13]
            0x40, // ttl
            0x11, // protocol
            0x82, 0xac, // checksum
            192, 168, 0, 1, // src
            192, 168, 0, 2, // dst
            // UDP packet
            0x96, 0xa6, // src_port
            0x0, 0x50, // dst_port
            0x0, 0xe, // length
            0xb1, 0xa1, 0x68, 0x6f, 0x6c, 0x6c, 0x61, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        ];
        let mut buf = Buffer::from_vec(512, raw);
        buf.reset();

        let (ip_p, ip_l) = ip::Packet::deserialize(&mut buf).unwrap();
        let mut data = Buffer::from_vec(512, ip_p.data);
        data.reset();
        let (udp_p, udp_l) = Datagram::deserialize(&mut data).unwrap();
        assert_eq!(udp_p.data, "holla".as_bytes());
    }
    
    #[test]
    fn test_udp_serialization() {
        let header = Header {
            src: 1,
            dst: 2,
            checksum: 3,
            length: 32,
        };
        let mut buf = Buffer::new(252);
        let n = header.serialize(&mut buf).unwrap();
        buf.reset();

        let (res, m) = Header::deserialize(&mut buf).unwrap();
        assert_eq!(n, m);
        assert_eq!(header, res);
    }
}
