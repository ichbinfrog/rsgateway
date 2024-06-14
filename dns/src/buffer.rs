use std::{
    fmt::Debug,
    mem::size_of,
    ops::{BitOrAssign, Sub},
};

use num_traits::{AsPrimitive, Num, PrimInt, Unsigned, Zero};

use super::error::PacketError;

pub const MAX_BUF_SIZE: usize = 512;
pub const MAX_LABEL_SIZE: usize = 63;

#[derive(Debug)]
pub struct PacketBuffer {
    pub buf: [u8; MAX_BUF_SIZE],
    pub pos: usize,
}

impl Default for PacketBuffer {
    fn default() -> Self {
        Self {
            buf: [0; MAX_BUF_SIZE],
            pos: 0,
        }
    }
}

impl PacketBuffer {
    pub fn set<T>(&mut self, pos: usize, val: T) -> Result<(), PacketError>
    where
        T: 'static + AsPrimitive<u8> + Unsigned + PrimInt,
        u8: AsPrimitive<T> + Num + Sized,
    {
        let n = size_of::<T>().sub(1);
        let mask: T = (0xFF_u8).as_();

        for i in 0..n {
            self.buf[pos + i] = ((val >> ((n - i) * 8)) & mask).as_();
        }
        Ok(())
    }

    pub async fn write<T>(&mut self, val: T) -> Result<(), PacketError>
    where
        T: 'static + AsPrimitive<u8> + Unsigned + PrimInt,
        u8: AsPrimitive<T> + Num + Sized,
    {
        if self.pos >= MAX_BUF_SIZE {
            return Err(PacketError::ContentTooLarge {
                max_size: MAX_BUF_SIZE,
            });
        }

        let n = size_of::<T>();
        let mask: T = (0xFF_u8).as_();
        for i in (0..n).rev() {
            self.buf[self.pos] = ((val >> (i * 8)) & mask).as_();
            self.pos += 1;
        }

        Ok(())
    }

    pub async fn write_qname(&mut self, qname: &str) -> Result<(), PacketError> {
        for label in qname.split('.') {
            let len = label.len();
            if len > MAX_LABEL_SIZE {
                return Err(PacketError::LabelTooLarge {
                    max_size: MAX_LABEL_SIZE,
                });
            }

            self.write::<u8>(len as u8).await?;
            self.buf[self.pos..self.pos + len].copy_from_slice(label.as_bytes());
            self.pos += len;
        }

        self.write(0_u8).await?;
        Ok(())
    }

    pub fn read<T>(&mut self) -> Result<T, PacketError>
    where
        T: 'static + Copy + Zero + BitOrAssign + std::ops::Shl<usize, Output = T> + Unsigned,
        u8: AsPrimitive<T> + Num + Sized,
    {
        if self.pos >= MAX_BUF_SIZE {
            return Err(PacketError::ContentTooLarge {
                max_size: MAX_BUF_SIZE,
            });
        }

        let n = size_of::<T>();
        let mut res: T = T::zero();
        for i in (0..n).rev() {
            res |= self.buf[self.pos].as_() << (8 * i);
            self.pos += 1;
        }

        Ok(res)
    }

    pub fn get(&self, i: usize) -> Result<u8, PacketError> {
        match self.buf.get(i) {
            Some(x) => Ok(*x),
            None => Err(PacketError::OutOfBound { index: i }),
        }
    }

    /*
    https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
    QNAME: a domain name represented as a sequence of labels, where
            each label consists of a length octet followed by that
            number of octets.  The domain name terminates with the
            zero length octet for the null label of the root.  Note
            that this field may be an odd number of octets; no padding is used.


    https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4

    In order to reduce the size of messages, the domain system utilizes a
    compression scheme which eliminates the repetition of domain names in a
    message.  In this scheme, an entire domain name or a list of labels at
    the end of a domain name is replaced with a pointer to a prior occurance
    of the same name.

    The pointer takes the form of a two octet sequence:

        +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        | 1  1|                OFFSET                   |
        +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+

    The first two bits are ones.  This allows a pointer to be distinguished
    from a label, since the label must begin with two zero bits because
    labels are restricted to 63 octets or less
    */
    pub fn read_qname(&mut self) -> Result<String, PacketError> {
        let mut res = String::new();
        let mut cur = self.pos;
        let mut delimiter = "";

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps = 0;

        loop {
            if jumps > max_jumps {
                return Err(PacketError::TooManyJumps);
            }

            let len = self.get(cur)?;

            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.pos = cur + 2;
                }

                let next = self.get(cur + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | next;
                cur = offset as usize;

                jumped = true;
                jumps += 1;
                continue;
            } else {
                cur += 1;
                if len == 0 {
                    break;
                }
                res.push_str(delimiter);
                res.push_str(&String::from_utf8_lossy(&self.buf[cur..cur + len as usize]));
                delimiter = ".";
                cur += len as usize;
            }
        }
        if !jumped {
            self.pos = cur;
        }
        Ok(res)
    }
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

impl From<u8> for ResponseCode {
    fn from(value: u8) -> Self {
        match value {
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFail,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            _ => ResponseCode::NoError,
        }
    }
}

// https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
#[derive(Debug, PartialEq)]
pub struct Header {
    pub id: u16,

    pub opcode: u8,

    pub query: bool,
    pub authoritative_answer: bool,
    pub truncated_message: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,

    pub zero: bool,
    pub authed_data: bool,
    pub checking_disabled: bool,

    pub response_code: ResponseCode,

    pub query_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_record_count: u16,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            id: 0,

            opcode: 0,

            query: false,
            authoritative_answer: false,
            truncated_message: false,
            recursion_available: false,
            recursion_desired: false,

            zero: true,
            checking_disabled: false,
            authed_data: false,

            response_code: ResponseCode::NoError,

            query_count: 0,
            answer_count: 0,
            authority_count: 0,
            additional_record_count: 0,
        }
    }
}

impl Header {
    pub async fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        buffer.write(self.id).await?;

        /*
         15 14 13 12 11 10  9  8  7  6  5 4   3  2  1  0
         7   6  5  4  3  2  1  0  7   6  5  4  3  2  1  0
        +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   |
        +--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+
        */
        buffer
            .write::<u8>(
                ((self.query as u8) << 7)
                    | (self.opcode << 3)
                    | ((self.authoritative_answer as u8) << 2)
                    | ((self.truncated_message as u8) << 1)
                    | self.recursion_desired as u8,
            )
            .await?;
        buffer
            .write::<u8>(
                ((self.recursion_available as u8) << 7)
                    | ((self.zero as u8) << 6)
                    | ((self.authed_data as u8) << 5)
                    | ((self.checking_disabled as u8) << 4)
                    | (self.response_code as u8),
            )
            .await?;

        buffer.write(self.query_count).await?;
        buffer.write(self.answer_count).await?;
        buffer.write(self.authority_count).await?;
        buffer.write(self.additional_record_count).await?;

        Ok(())
    }
}

impl TryFrom<&mut PacketBuffer> for Header {
    type Error = PacketError;

    fn try_from(buffer: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let id = buffer.read()?;
        let flags = buffer.read::<u16>()?;
        let left = (flags >> 8) as u8;
        let right = (flags & 0xFF) as u8;

        Ok(Self {
            id,
            recursion_desired: (left & (1 << 0)) > 0,
            truncated_message: (left & (1 << 1)) > 0,
            authoritative_answer: (left & (1 << 2)) > 0,
            query: (left & (1 << 7)) > 0,
            opcode: (left >> 3) & 0x0F,

            recursion_available: (right & (1 << 7)) > 0,
            zero: (right & (1 << 6)) > 0,
            authed_data: (right & (1 << 5)) > 0,
            checking_disabled: (right & (1 << 4)) > 0,
            response_code: ResponseCode::from(right & 0x0F),

            query_count: buffer.read()?,
            answer_count: buffer.read()?,
            authority_count: buffer.read()?,
            additional_record_count: buffer.read()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tokio::net::UdpSocket;

    use crate::{
        packet::Packet,
        question::{Question, QuestionClass, QuestionKind},
    };

    use super::*;
    use rstest::*;
    use tempfile::tempfile;

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
            opcode: 15,
            query: true,
            authoritative_answer: true,
            truncated_message: true,
            recursion_desired: true,
            recursion_available: true,
            zero: true,
            authed_data: true,
            checking_disabled: true,
            response_code: ResponseCode::NoError,
            query_count: u16::MAX,
            answer_count: u16::MAX,
            authority_count: u16::MAX,
            additional_record_count: u16::MAX,
        }
    )]
    fn test_header_read(#[case] input: &[u8], #[case] expected: Header) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..12].copy_from_slice(input);
        assert_eq!(Header::try_from(&mut pb).unwrap(), expected);
    }

    #[rstest]
    #[case(
        Header {
            id: u16::MAX,
            opcode: 15,
            query: true,
            authoritative_answer: true,
            truncated_message: true,
            recursion_desired: true,
            recursion_available: true,
            zero: true,
            authed_data: true,
            checking_disabled: true,
            response_code: ResponseCode::NoError,
            query_count: u16::MAX,
            answer_count: u16::MAX,
            authority_count: u16::MAX,
            additional_record_count: u16::MAX,
        }
    )]
    #[tokio::test]
    async fn test_header_write(#[case] input: Header) {
        let mut pb = PacketBuffer::default();
        assert!(input.write(&mut pb).await.is_ok());
        pb.pos = 0;            0

        assert!(Header::try_from(&mut pb).is_ok_and(|x| x == input));
    }

    #[rstest]
    #[case(
        &[
            3, b'w', b'w', b'w',
            7, b'h', b't', b't', b'p', b'b', b'i', b'n',
            3, b'o', b'r', b'g',
        ],
        "www.httpbin.org",
    )]
    #[case(
        &[
            3, b'w', b'w', b'w',
            0xC0, 10,
            0, 0, 0, 0,
            3, b'f', b'o', b'o',
        ],
        "www.foo",
    )]
    fn test_qname_read(#[case] input: &[u8], #[case] expected: &str) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..input.len()].copy_from_slice(input);

        let res = pb.read_qname().unwrap();
        assert_eq!(res, expected.to_string());
    }

    #[rstest]
    #[case("test.google.com")]
    #[case("www")]
    #[tokio::test]
    async fn test_qname_write(#[case] input: &str) {
        let mut pb = PacketBuffer::default();
        assert!(pb.write_qname(input).await.is_ok());
        pb.pos = 0;
        let res = pb.read_qname().unwrap();
        assert_eq!(res, input);
    }

    #[tokio::test]
    async fn test_wire() {
        let server = ("8.8.8.8", 53);
        let socket = UdpSocket::bind(("0.0.0.0", 43210)).await.unwrap();

        let packet = Packet {
            header: Header {
                id: 30000,
                query_count: 1,
                recursion_desired: true,
                ..Default::default()
            },
            questions: Some(vec![Question {
                name: "google.com".to_string(),
                kind: QuestionKind::AAAA,
                class: QuestionClass::IN,
            }]),
            answers: None,
            authorities: None,
            resources: None,
        };
        let mut req: PacketBuffer = PacketBuffer::default();
        packet.write(&mut req).await.unwrap();

        let mut file = tempfile().unwrap();
        file.write_all(&req.buf[0..req.pos]).unwrap();

        socket.send_to(&req.buf[0..req.pos], server).await.unwrap();
        let mut res = PacketBuffer::default();
        socket.recv_from(&mut res.buf).await.unwrap();

        println!("{:?}", Packet::try_from(&mut res));
    }
}
