use arbitrary_int::{u3, u4};
use bitarray::{
    buffer::{self, Buffer, Error},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};

pub const MAX_BUF_SIZE: usize = 512;
pub const MAX_LABEL_SIZE: usize = 63;

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
            Self: Sized {

        let (code, code_l) = buf.read_arbitrary_u8::<u3>()?;
        let res = match code.value() {
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFail,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NoError
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
        buf.push_arbitrary_u8::<u3>(res)
    }
}


#[derive(Debug)]
pub struct QName(String);

impl Serialize for QName {
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut qname_l: usize = 0;
        for label in self.0.split('.') {
            let n = label.len();
            if n > MAX_LABEL_SIZE {
                return Err(Error::Overflow { size: n, max: MAX_LABEL_SIZE })
            }

            qname_l += buf.push_primitive::<u8>(n as u8)?;
        
            let index = buf.pos();
            // udp packets are byte aligned so we can do this unsafe op
            buf.data[index.pos..index.pos+n].copy_from_slice(label.as_bytes());
            qname_l += buf.skip(n * buffer::BYTE_SIZE)?;
        }

        Ok(qname_l)
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
    #[case(
        &[
            0xc2,0x39,
            0x81,0x80,0x00,0x01,
            0x00,0x01,0x00,0x00,0x00,0x00,
        ],
    )]
    fn test_packet_parsing(#[case] input: &[u8]) {
        let mut buf = Buffer::from_vec(MAX_BUF_SIZE, input.to_vec());
        buf.reset();

        let (header, header_l) = Header::deserialize(&mut buf).unwrap();
        println!("{:?}", header);
    }
}