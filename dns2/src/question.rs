use std::net::{Ipv4Addr, Ipv6Addr};

use bitarray::{
    buffer::{self, Buffer, Error},
    serialize::{self, Deserialize, Serialize},
};
use bitarray_derive::{Deserialize, Serialize};
type DeserializeError = DnsError;
type SerializeError = DnsError;

use crate::{error::DnsError, packet};

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Eq, Ord, Serialize)]
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

// impl Serialize for QuestionKind {
//     type Err = DnsError;
//     fn serialize(&self, buf: &mut bitarray::buffer::Buffer) -> Result<usize, Self::Err> {
//         let res: u16 = match self {
//             Self::A => 1,
//             Self::NS => 2,
//             Self::CNAME => 5,
//             Self::SOA => 6,
//             Self::PTR => 12,
//             Self::MX => 15,
//             Self::AAAA => 28,
//         };
//         Ok(buf.push_primitive::<u16>(res)?)
//     }
// }

// impl Deserialize for QuestionKind {
//     type Err = DnsError;
//     fn deserialize(buf: &mut bitarray::buffer::Buffer) -> Result<(Self, usize), Self::Err>
//     where
//         Self: Sized,
//     {
//         let (kind, kind_l) = buf.read_primitive::<u16, 2>()?;
//         match kind {
//             1 => Ok((Self::A, kind_l)),
//             2 => Ok((Self::NS, kind_l)),
//             5 => Ok((Self::CNAME, kind_l)),
//             6 => Ok((Self::SOA, kind_l)),
//             12 => Ok((Self::PTR, kind_l)),
//             15 => Ok((Self::MX, kind_l)),
//             28 => Ok((Self::AAAA, kind_l)),
//             _ => Err(DnsError::Unimplemented {
//                 sub: "question_kind",
//                 reason: "unknown question kind",
//             }),
//         }
//     }
// }

#[derive(Debug, PartialEq, Copy, Clone, Serialize)]
#[bitarray(repr(u16))]
pub enum QuestionClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

// impl Serialize for QuestionClass {
//     type Err = DnsError;
//     fn serialize(&self, buf: &mut bitarray::buffer::Buffer) -> Result<usize, Self::Err> {
//         let res: u16 = match self {
//             QuestionClass::IN => 1,
//             QuestionClass::CS => 2,
//             QuestionClass::CH => 3,
//             QuestionClass::HS => 4,
//         };
//         Ok(buf.push_primitive(res)?)
//     }
// }

// impl Deserialize for QuestionClass {
//     type Err = DnsError;
//     fn deserialize(buf: &mut bitarray::buffer::Buffer) -> Result<(Self, usize), Self::Err>
//     where
//         Self: Sized,
//     {
//         let (class, class_l) = buf.read_primitive::<u16, 2>()?;
//         match class {
//             1 => Ok((QuestionClass::IN, class_l)),
//             2 => Ok((QuestionClass::CS, class_l)),
//             3 => Ok((QuestionClass::CH, class_l)),
//             4 => Ok((QuestionClass::HS, class_l)),
//             _ => Err(DnsError::Unimplemented {
//                 sub: "question_class",
//                 reason: "unimplemented question class",
//             }),
//         }
//     }
// }

// #[derive(Debug, PartialEq, Serialize)]
// pub enum Record {
//     UNKNOWN {
//         question: Question,
//         ttl: u32,
//         rd_length: u16,
//     },
//     A {
//         question: Question,
//         addr: Ipv4Addr,
//         ttl: u32,
//         rd_length: u16,
//     },
//     NS {
//         question: Question,
//         host: String,
//         ttl: u32,
//         rd_length: u16,
//     },
//     CNAME {
//         question: Question,
//         host: String,
//         ttl: u32,
//         rd_length: u16,
//     },
//     SOA {
//         question: Question,
//         mname: String,
//         rname: String,
//         serial: u32,
//         refresh: u32,
//         retry: u32,
//         expire: u32,
//         minimum: u32,
//         ttl: u32,
//         rd_length: u16,
//     },
//     PTR {
//         question: Question,
//         host: String,
//         ttl: u32,
//         rd_length: u16,
//     },
//     MX {
//         question: Question,
//         preference: u16,
//         exchange: String,
//         ttl: u32,
//         rd_length: u16,
//     },
//     AAAA {
//         question: Question,
//         addr: Ipv6Addr,
//         ttl: u32,
//         rd_length: u16,
//     },
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// pub struct Question {
//     name: packet::QName,
//     kind: QuestionKind,
//     class: QuestionClass,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use packet::QName;
//     use rstest::*;

//     #[rstest]
//     #[case(
//         &[
//             6, b'g', b'o', b'o', b'g', b'l', b'e',
//             3, b'c', b'o', b'm', 0,
//             0, 1,
//             0, 1,
//             0
//         ],
//         Question {
//             name: QName("google.com".to_string()),
//             kind: QuestionKind::A,
//             class: QuestionClass::IN,
//         },
//     )]
//     #[case(
//         &[
//             6, b'g', b'o', b'o', b'g', b'l', b'e',
//             3, b'c', b'o', b'm', 0,
//             0, 5,
//             0, 1,
//             0
//         ],
//         Question {
//             name: QName("google.com".to_string()),
//             kind: QuestionKind::CNAME,
//             class: QuestionClass::IN,
//         },
//     )]
//     fn test_question_parse(#[case] input: &[u8], #[case] expected: Question) {
//         let mut buf = Buffer::from_vec(input.to_vec());
//         buf.reset();
//         let (question, _) = Question::deserialize(&mut buf).unwrap();
//         assert_eq!(question, expected);
//     }
// }
