use std::net::Ipv4Addr;

use arbitrary_int::{u13, u3, u4};
use bitarray::{
    buffer::{self},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Prefix {
    tun_flags: u16,
    tun_proto: u16,
    version: u4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    ihl: u4,
    tos: u8,
    length: u16,

    ident: u16,
    flags: u3,
    offset: u13,

    ttl: u8,
    protocol: u8,
    checksum: u16,

    src: Ipv4Addr,
    dst: Ipv4Addr,
}

#[cfg(test)]
pub mod tests {
    use buffer::Buffer;

    use super::*;

    #[test]
    fn test_ip_packet_parsing() {
        let raw: Vec<u8> = vec![
            0x0, 0x0, 
            0x8, 0x0, 
            0x45, 0x0, 0x0, 0x54, 0x44, 0x1d, 0x40, 0x0, 0x40, 
            0x1, 0x75, 0x38, 0xc0, 0xa8, 0x0, 0x1, 0xc0, 0xa8, 
            0x0, 0x2, 0x8, 0x0, 0x48, 0x8a, 0x0, 0x9, 0x0, 
            0x1, 0xc6, 0xfd, 0x6a, 0x66, 0x0, 0x0, 0x0, 0x0, 
            0xb0, 0x34, 0xf, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 
            0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 
            0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 
        ];

        let mut buf = Buffer::from_vec(256, raw);
        buf.reset();

        let (prefix, m) = Prefix::deserialize(&mut buf).unwrap();
        println!("{:?}", prefix);

        match prefix.version.value() {
            4 => {
                let (packet, _) = Packet::deserialize(&mut buf).unwrap();
                println!("{:?}", packet);
            }
            _ => unimplemented!("ip version not implemented")
        }
    }
}
