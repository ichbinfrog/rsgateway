use arbitrary_int::{u14, u2, u3, u4, u6, Number};
use bitarray::{
    buffer::{self, Buffer, Error},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

use crate::error::DnsError;

pub const MAX_BUF_SIZE: usize = 512;
pub const MAX_LABEL_SIZE: usize = 63;
pub const MAX_QNAME_COMPRESSION_JUMPS: usize = 5;

/*
https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
                                    1  1  1  1  1  1
      0  1  2  3  4  5  6  7  8  9  0  1  2  3  4  5
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                      ID                       |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    QDCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ANCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    NSCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
    |                    ARCOUNT                    |
    +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

*/

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Header {
    id: u16,
    qr: bool,
    opcode: u4,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    zero: u3,
    rcode: ResponseCode,

    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResponseCode {
    NoError = 0,
    FormatError = 1,
    ServerFail = 2,
    NameError = 3,
    NotImplemented = 4,
    Refused = 5,
}

impl Deserialize for ResponseCode {
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (code, code_l) = buf.read_arbitrary_u8::<u3>()?;
        let res = match code.value() {
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFail,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NoError,
        };
        Ok((res, code_l))
    }
}

impl Serialize for ResponseCode {
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let res = match self {
            ResponseCode::FormatError => u3::new(1),
            ResponseCode::ServerFail => u3::new(2),
            ResponseCode::NameError => u3::new(3),
            ResponseCode::NotImplemented => u3::new(4),
            ResponseCode::Refused => u3::new(5),
            ResponseCode::NoError => u3::new(0),
        };
        Ok(buf.push_arbitrary_u8::<u3>(res)?)
    }
}

#[derive(Debug, PartialEq)]
pub struct QName(pub(crate) String);

impl Serialize for QName {
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut qname_l: usize = 0;
        for label in self.0.split('.') {
            let n = label.len();
            if n > MAX_LABEL_SIZE {
                return Err(Error::Custom {
                    reason: "label too large".to_string(),
                });
            }

            qname_l += buf.push_primitive::<u8>(n as u8)?;

            let index = buf.pos()?;
            if buf.is_aligned()? {
                // udp packets are byte aligned so we can do this unsafe op
                buf.data[index.pos..index.pos + n].copy_from_slice(label.as_bytes());
                qname_l += buf.skip(n * buffer::BYTE_SIZE)?;
            }
        }

        Ok(qname_l)
    }
}

impl Deserialize for QName {
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let mut res = String::new();
        let mut delimeter = "";
        let mut jumps: usize = 0;
        let jump_flag: u8 = 0b11;

        loop {
            if jumps > MAX_QNAME_COMPRESSION_JUMPS {
                return Err(Error::Custom {
                    reason: "too many jumps".to_string(),
                });
            }

            let (jump, _) = buf.read_arbitrary_u8::<u2>()?;
            if jump.value() == jump_flag {
                let (offset, _) = buf.read_arbitrary_u16::<u14>()?;
                let offset = offset.value();
                buf.seek(offset as usize * buffer::BYTE_SIZE);
                jumps += 1;
            } else {
                let (len, _) = buf.read_arbitrary_u8::<u6>()?;
                let len = len.value();
                if len == 0 {
                    break;
                }
                res.push_str(delimeter);
                let pos = buf.pos()?.pos;
                res.push_str(&String::from_utf8_lossy(&buf.data[pos..pos + len as usize]));
                delimeter = ".";
                buf.skip(buffer::BYTE_SIZE * len as usize)?;
            }
        }

        Ok((QName(res), 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        &[
            u8::MAX, u8::MAX,
            u8::MAX, u8::MAX,
            u8::MAX, u8::MAX,
            u8::MAX, u8::MAX,
            u8::MAX, u8::MAX,
            u8::MAX, u8::MAX,
        ],
        Header { 
            id: u16::MAX, 
            qr: true, opcode: u4::new(15), 
            aa: true, tc: true, 
            rd: true, ra: true, 
            zero: u3::new(7), 
            rcode: ResponseCode::NoError, 
            qd_count: u16::MAX, 
            an_count: u16::MAX, 
            ns_count: u16::MAX, 
            ar_count: u16::MAX,
        }
    )]
    fn test_packet_parsing(#[case] input: &[u8], #[case] expected: Header) {
        let mut buf = Buffer::from_vec(input.to_vec());
        buf.reset();

        let (header, _) = Header::deserialize(&mut buf).unwrap();
        assert_eq!(header, expected);
    }

    #[rstest]
    #[case(
        &[
            1, b'f', 
            3, b'i', b's', b'i',
            4, b'a', b'r', b'p', b'a', 0,
            0, 0, 0, 0, 0, 0, 0, 0, 
            3, b'f', b'o', b'o',
            0xC0, 0,
            0,
            0xC0, 6,
            0
        ],
        20,
        27,
        "f.isi.arpa",
        "foo.f.isi.arpa",
        "arpa",
    )]
    #[case(
        &[
            1, b'f', 
            3, b'i', b's', b'i',
            4, b'a', b'r', b'p', b'a', 0,
            0, 0, 0, 0, 0, 0, 0, 0, 
            3, b'f', b'o', b'o',
            0xC0, 0,
            0,
            0xC0, 20,
            0
        ],
        20,
        27,
        "f.isi.arpa",
        "foo.f.isi.arpa",
        "foo.f.isi.arpa",
    )]
    fn test_qname_compression_read(
        #[case] input: &[u8],
        #[case] first_offset: usize,
        #[case] second_offset: usize,
        #[case] first_expected: &str,
        #[case] second_expected: &str,
        #[case] third_expected: &str,
    ) {
        let mut buf = Buffer::from_vec(input.to_vec());
        buf.reset();
        let (first, _) = QName::deserialize(&mut buf).unwrap();
        assert_eq!(first.0, first_expected.to_string());

        buf.seek(first_offset * buffer::BYTE_SIZE);
        let (second, _) = QName::deserialize(&mut buf).unwrap();
        assert_eq!(second.0, second_expected.to_string());

        buf.seek(second_offset * buffer::BYTE_SIZE);
        let (third, _) = QName::deserialize(&mut buf).unwrap();
        assert_eq!(third.0, third_expected.to_string());
    }

    #[rstest]
    #[case(
        &[
            0xC0, 3, 0,
            0xC0, 0, 0,
        ],
        DnsError::TooManyJumps {
            max: MAX_QNAME_COMPRESSION_JUMPS
        },
    )]
    #[case(
        &[
            0xC0, 3, 0,
            3, b'f', b'o', b'o', 
            0xC0, 0, 0,
        ],
        DnsError::TooManyJumps {
            max: MAX_QNAME_COMPRESSION_JUMPS
        }
    )]
    #[case(
        &[
            0xC0, 0, 0
        ],
        DnsError::TooManyJumps {
            max: MAX_QNAME_COMPRESSION_JUMPS
        }
    )]
    #[case(
        &[
            0xC0, 3, 0,
            0xC0, 6, 0,
            0xC0, 9, 0,
            0xC0, 12, 0,
            0xC0, 15, 0,
            0xC0, 18, 0,
            0, 0, 
        ],
        DnsError::TooManyJumps {
            max: MAX_QNAME_COMPRESSION_JUMPS
        }
    )]
    #[case(
        &[
            0xC0, 255, 0,
        ],
        DnsError::IOError(Error::OutOfRange { size: 24, pos: 2048 })
    )]
    fn test_qname_compression_edge_cases(#[case] input: &[u8], #[case] err: DnsError) {
        let mut buf = Buffer::from_vec(input.to_vec());
        buf.reset();
        let res = QName::deserialize(&mut buf);
        // assert_eq!(res.unwrap_err(), err,);
    }
}
