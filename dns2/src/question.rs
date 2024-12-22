use std::net::{Ipv4Addr, Ipv6Addr};

use bitarray::buffer::{Buffer, Error};

use bitarray::decode::Decoder;
use bitarray::encode::Encoder;
use bitarray_derive::{Decode, Encode};

use crate::packet::QName;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Eq, Ord, Encode, Decode)]
#[bitarray(repr(u16))]
pub enum QuestionKind {
    A = 1,
    NS = 2,
    CNAME = 5,
    SOA = 6,

    PTR = 12,
    MX = 15,
    AAAA = 28,
}

#[derive(Debug, PartialEq, Copy, Clone, Encode, Decode)]
#[bitarray(repr(u16))]
pub enum QuestionClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Question {
    name: QName,
    kind: QuestionKind,
    class: QuestionClass,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[bitarray(repr(u16))]
#[repr(u16)]
pub enum RecordType {
    A {
        addr: Ipv4Addr,
    } = 1,
    AAA {
        addr: Ipv6Addr,
    } = 28,
    NS {
        host: QName,
    } = 2,
    CNAME {
        host: QName,
    } = 5,
    SOA {
        mname: QName,
        rname: QName,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    } = 6,
    PTR {
        host: QName,
    } = 12,
    MX {
        preference: u16,
        exchange: QName,
    } = 15,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct RecordHeader {
    question: Question,
    ttl: u32,
    rd_length: u16,
}

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Record {
    header: RecordHeader,

    // #[bitarray(variant(header.question.kind))]
    // data: RecordType,
}

// impl Decoder for Record {
//     fn decode(buf: &mut Buffer) -> Result<(Self, usize), Error>
//         where
//             Self: Sized {
//         let (question, question_l) = Question::decode(buf)?;
//         match question.kind {
//             QuestionKind::A => {
//             }
//         }
//     }
// }

// impl Encoder for Record {
//     fn encode(&self, buf: &mut Buffer) -> Result<usize, Error> {

//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        &[
            6, b'g', b'o', b'o', b'g', b'l', b'e',
            3, b'c', b'o', b'm', 0,
            0, 1,
            0, 1,
            0
        ],
        Question {
            name: QName("google.com".to_string()),
            kind: QuestionKind::A,
            class: QuestionClass::IN,
        },
    )]
    #[case(
        &[
            6, b'g', b'o', b'o', b'g', b'l', b'e',
            3, b'c', b'o', b'm', 0,
            0, 5,
            0, 1,
            0
        ],
        Question {
            name: QName("google.com".to_string()),
            kind: QuestionKind::CNAME,
            class: QuestionClass::IN,
        },
    )]
    fn test_question_parse(#[case] input: &[u8], #[case] expected: Question) {
        let mut buf = Buffer::from_vec(input.to_vec());
        buf.reset();
        let (question, _) = Question::decode(&mut buf).unwrap();
        assert_eq!(question, expected);
    }
}
