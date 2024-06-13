use arbitrary_int::u4;
use bitarray::{
    buffer::{self},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    flags: u16,
    proto: u16,

    version: u4,
    ihl: u4,
    tos: u8,
    ident: u16,

    frag: u16, // [flags;3 , offset:13]
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src: u32,
    dst: u32,
}

#[cfg(test)]
pub mod tests {
    use buffer::Buffer;
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_ip_packet_parsing() {
        let raw: Vec<u8> = vec![
            0, 0, 134, 221, 96, 0, 0, 0, 0, 8, 58, 255, 254, 128, 0, 0, 0, 0, 0, 0, 202, 55, 42,
            235, 206, 40, 241, 55, 255, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 133, 0, 200,
            179,
        ];

        let mut buf = Buffer::from_vec(256, raw);
        buf.reset();

        let (res, m) = Packet::deserialize(&mut buf).unwrap();
        println!("{:?}", res);

        println!("{:?}", Ipv4Addr::from_bits(res.src));
        println!("{:?}", Ipv4Addr::from_bits(res.dst));
    }
}
