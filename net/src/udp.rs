use bitarray::buffer::{Buffer, Error, BYTE_SIZE};

use bitarray::decode::Decoder;
use bitarray::encode::Encoder;
use bitarray_derive::{Decode, Encode};

// An UDP Packet as defined in [RFC-768](https://datatracker.ietf.org/doc/html/rfc768)
#[derive(Decode, Encode, PartialEq, Debug)]
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

impl Decoder for Datagram {
    fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (header, header_l) = Header::decode(buf)?;
        let data = buf.read_exact_n(header.length as usize - (header_l / BYTE_SIZE) - 1)?;
        let data_l = data.len();
        Ok((Self { header, data }, header_l + data_l))
    }
}

#[cfg(test)]
mod tests {

    use Buffer;

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
        let mut buf = Buffer::from_vec(raw);
        buf.reset();

        let (ip_p, ip_l) = ip::Packet::decode(&mut buf).unwrap();
        let mut data = Buffer::from_vec(ip_p.data);
        data.reset();
        let (udp_p, udp_l) = Datagram::decode(&mut data).unwrap();
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
        let n = header.encode(&mut buf).unwrap();
        buf.reset();

        let (res, m) = Header::decode(&mut buf).unwrap();
        assert_eq!(n, m);
        assert_eq!(header, res);
    }
}
