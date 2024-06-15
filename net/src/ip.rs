use std::{default, net::Ipv4Addr};

use arbitrary_int::{u13, u24, u3, u4};
use bitarray::{
    buffer::{self, Buffer, Error},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};
type DeserializeError = Error;
type SerializeError = Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Prefix {
    tun_flags: u16,
    tun_proto: u16,
    version: u4,
}

#[derive(Debug, Default)]
pub struct Packet {
    header: Header,
    pub data: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct Header {
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

    #[bitarray(condition(offset.value() > 0))]
    options: OptList,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            ihl: u4::new(0),
            tos: 0,
            length: 0,
            ident: 0,
            flags: u3::new(0),
            offset: u13::new(0),
            ttl: 0,
            protocol: 0,
            checksum: 0,
            options: OptList::default(),
            src: Ipv4Addr::new(0, 0, 0, 0),
            dst: Ipv4Addr::new(0, 0, 0, 0),
        }
    }
}

impl Deserialize for Packet {
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), buffer::Error>
    where
        Self: Sized,
    {
        let (prefix, _) = Prefix::deserialize(buf)?;
        match prefix.version.value() {
            4 => {
                let (header, header_l) = Header::deserialize(buf)?;
                let data =
                    buf.read_exact_n(header.length as usize - (header_l / buffer::BYTE_SIZE))?;
                let data_l = data.len();
                Ok((Self { header, data }, header_l + data_l))
            }
            _ => Ok((
                Self {
                    ..Default::default()
                },
                0,
            )),
        }
    }
}

#[derive(Debug, Default)]
pub struct OptList(Vec<Opt>);

impl Deserialize for OptList {
    fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), buffer::Error>
    where
        Self: Sized,
    {
        let mut res = Vec::<Opt>::new();
        let mut i = 0;
        loop {
            let (opt, n) = Opt::deserialize(buf)?;
            i += n;
            match opt {
                Opt::EndOfList => {
                    res.push(opt);
                    break;
                }
                _ => res.push(opt),
            }
        }
        Ok((OptList(res), i))
    }
}

#[derive(Debug)]
pub enum Opt {
    EndOfList,
    NoOp,
    Security {
        s: u16,
        c: u16,
        h: u16,
        tcc: u24,
    },
    LooseSrcRouting {
        ptr: u8,
        addr: Option<Vec<Ipv4Addr>>,
    },
    StrictSrcRouting {
        ptr: u8,
        addr: Option<Vec<Ipv4Addr>>,
    },
    RecordRoute {
        ptr: u8,
        addr: Option<Vec<Ipv4Addr>>,
    },
    StreamID {
        id: u16,
    },
    InternetTimestamp {
        ptr: u8,
    },
}

impl Deserialize for Opt {
    fn deserialize(buf: &mut buffer::Buffer) -> Result<(Self, usize), buffer::Error>
    where
        Self: Sized,
    {
        let (ty, n) = buf.read_primitive::<u8, 1>()?;
        let copied = (ty >> 7) != 0;
        let class = ty >> 5;
        let number = ty & 0b11111;
        match (class, number) {
            (0, 0) => return Ok((Opt::EndOfList, n)),
            (0, 1) => return Ok((Opt::NoOp, n)),
            (0, 2) => {
                buf.skip(2)?;
                let (s, s_l) = buf.read_primitive::<u16, 2>()?;
                let (c, c_l) = buf.read_primitive::<u16, 2>()?;
                let (h, h_l) = buf.read_primitive::<u16, 2>()?;
                let (tcc, tcc_l) = buf.read_arbitrary_u32::<u24>()?;
                return Ok((Opt::Security { s, c, h, tcc }, n + s_l + c_l + h_l + tcc_l));
            }
            // (0, 3) => {}
            // (0, 7) => {}
            (0, 8) => {
                buf.skip(2)?;
                let (id, id_l) = buf.read_primitive::<u16, 2>()?;
                return Ok((Opt::StreamID { id }, n + id_l));
            }
            // (2, 9) => {}
            _ => {
                unimplemented!(
                    "ip options not implemented for (copied={},class={},number={})",
                    copied,
                    class,
                    number
                );
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use buffer::Buffer;

    use super::*;

    #[test]
    fn test_ip_packet_option_parsing() {
        0x54;
        let raw: Vec<u8> = vec![
            0x0, 0x0, // tun_flags
            0x8, 0x0,  // tun_proto
            0x45, // version | ihl
            0x0,  // tos
            0x0, 0x54, // length
            0x44, 0x1d, // ident
            0x40, 0x5,  // flags[3] | offset[13]
            0x40, // ttl
            0x1,  // protocol
            0x75, 0x38, // checksum
            192, 168, 0, 1, // src
            192, 168, 0, 2,   // dst
            0x1, // noOp
            0x0, // eol
        ];

        let mut buf = Buffer::from_vec(raw);
        buf.reset();
        let (ip_p, ip_l) = Packet::deserialize(&mut buf).unwrap();
    }

    #[test]
    fn test_ip_packet_parsing() {
        let raw: Vec<u8> = vec![
            0x0, 0x0, 0x8, 0x0, 0x45, 0x0, 0x0, 0x54, 0x44, 0x1d, 0x40, 0x0, 0x40, 0x1, 0x75, 0x38,
            192, 168, 0, 1, 192, 168, 0, 2, 0x8, 0x0, 0x48, 0x8a, 0x0, 0x9, 0x0, 0x1, 0xc6, 0xfd,
            0x6a, 0x66, 0x0, 0x0, 0x0, 0x0, 0xb0, 0x34, 0xf, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x11,
            0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d,
            0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
        ];

        let mut buf = Buffer::from_vec(raw);
        buf.reset();
        let (ip_p, ip_l) = Packet::deserialize(&mut buf).unwrap();
    }
}
