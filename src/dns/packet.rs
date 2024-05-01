use std::{error::Error, mem::size_of, ops::{BitOrAssign, Shl}, process::Output};

pub const MAX_BUF_SIZE: usize = 512;

#[derive(Debug)]
pub enum PacketError {
    ContentTooLarge { max_size: usize },
    OutOfBound { index: usize },
    InvalidBound { start: usize, end: usize },
    TooManyJumps,
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for PacketError {}

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
    pub fn write(&mut self, val: u8) -> Result<(), PacketError> {
        if self.pos >= MAX_BUF_SIZE {
            return Err(PacketError::ContentTooLarge {
                max_size: MAX_BUF_SIZE,
            })
        }

        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u16(&mut self, val: u8) -> Result<(), PacketError> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_u32(&mut self, val: u8) -> Result<(), PacketError> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write((val & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<(), PacketError> {
        for label in qname.split('.') {
            let len = label.len();
            self.write(len as u8)?;
        }

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8, PacketError> {
        if self.pos >= MAX_BUF_SIZE {
            return Err(PacketError::ContentTooLarge {
                max_size: MAX_BUF_SIZE,
            });
        }

        let res = self.buf[self.pos];
        self.pos += 1;
        Ok(res)
    }

    pub fn read_u16(&mut self) -> Result<u16, PacketError> {
        let res = ((self.read()? as u16) << 8) | ((self.read()? as u16) << 0);
        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32, PacketError> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);
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

                println!("{:?}, {:?}, {:?}", len, next, offset);
                continue;
            } else {
                cur += 1;
                if len == 0 {
                    break;
                }
                res.push_str(&delimiter);
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

#[derive(Debug, PartialEq)]
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
            0 | _ => ResponseCode::NoError,
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

   pub  zero: bool,
    pub authed_data: bool,
    pub checking_disabled: bool,

   pub  response_code: ResponseCode,

   pub  query_count: u16,
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

impl TryFrom<&mut PacketBuffer> for Header {
    type Error = PacketError;

    fn try_from(buffer: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let id = buffer.read_u16()?;
        let flags = buffer.read_u16()?;
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
            checking_disabled: (right & (1 << 4)) > 0,
            authed_data: (right & (1 << 5)) > 0,
            zero: (right & (1 << 6)) > 0,
            response_code: ResponseCode::from(right & 0x0F),

            query_count: buffer.read_u16()?,
            answer_count: buffer.read_u16()?,
            authority_count: buffer.read_u16()?,
            additional_record_count: buffer.read_u16()?,
        })
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
        &[
            3, 'w' as u8, 'w' as u8, 'w' as u8,
            7, 'h' as u8, 't' as u8, 't' as u8, 'p' as u8, 'b' as u8, 'i' as u8, 'n' as u8,
            3, 'o' as u8, 'r' as u8, 'g' as u8,
        ],
        "www.httpbin.org",
    )]
    #[case(
        &[
            3, 'w' as u8, 'w' as u8, 'w' as u8,
            0xC0, 10,
            0, 0, 0, 0,
            3, 'f' as u8, 'o' as u8, 'o' as u8,
        ],
        "www.foo",
    )]
    fn test_qname_read(#[case] input: &[u8], #[case] expected: &str) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..input.len()].copy_from_slice(input);

        let res = pb.read_qname().unwrap();
        assert_eq!(res, expected.to_string());
    }

    #[test]
    fn test_sizing() {
        let mut pb = PacketBuffer::default();
        pb.write_sized(8 as u8);
        pb.write_sized(8 as u16);
    }
}
