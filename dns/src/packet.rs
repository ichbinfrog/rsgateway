use std::{
    net::{Ipv4Addr, Ipv6Addr},
};

use tokio::net::UdpSocket;

use crate::error::LookupError;

use super::{
    buffer::{Header, PacketBuffer},
    error::PacketError,
    question::{Question, QuestionKind},
};

// https://www.internic.net/domain/named.root
const NAMED_ROOT: [Ipv4Addr; 13] = [
    Ipv4Addr::new(198, 41, 0, 4),
    Ipv4Addr::new(170, 247, 170, 2),
    Ipv4Addr::new(192, 33, 4, 12),
    Ipv4Addr::new(199, 7, 91, 13),
    Ipv4Addr::new(192, 203, 230, 10),
    Ipv4Addr::new(192, 5, 5, 241),
    Ipv4Addr::new(192, 112, 36, 4),
    Ipv4Addr::new(198, 97, 190, 53),
    Ipv4Addr::new(192, 36, 148, 17),
    Ipv4Addr::new(192, 58, 128, 30),
    Ipv4Addr::new(193, 0, 14, 129),
    Ipv4Addr::new(199, 7, 83, 42),
    Ipv4Addr::new(202, 12, 27, 33),
];

#[derive(Debug, PartialEq)]
pub enum Record {
    UNKNOWN {
        question: Question,
        ttl: u32,
        rd_length: u16,
    },
    A {
        question: Question,
        addr: Ipv4Addr,
        ttl: u32,
        rd_length: u16,
    },
    NS {
        question: Question,
        host: String,
        ttl: u32,
        rd_length: u16,
    },
    CNAME {
        question: Question,
        host: String,
        ttl: u32,
        rd_length: u16,
    },
    SOA {
        question: Question,
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
        ttl: u32,
        rd_length: u16,
    },
    PTR {
        question: Question,
        host: String,
        ttl: u32,
        rd_length: u16,
    },
    MX {
        question: Question,
        preference: u16,
        exchange: String,
        ttl: u32,
        rd_length: u16,
    },
    AAAA {
        question: Question,
        addr: Ipv6Addr,
        ttl: u32,
        rd_length: u16,
    },
}

impl TryFrom<&mut PacketBuffer> for Record {
    type Error = PacketError;

    #[inline]
    fn try_from(buf: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let question = Question::try_from(buf as &mut PacketBuffer)?;
        let ttl = buf.read::<u32>()?;
        let rd_length = buf.read::<u16>()?;

        match question.kind {
            QuestionKind::A => {
                let raw = buf.read::<u32>()?;

                let addr = Ipv4Addr::new(
                    ((raw >> 24) & 0xFF) as u8,
                    ((raw >> 16) & 0xFF) as u8,
                    ((raw >> 8) & 0xFF) as u8,
                    (raw & 0xFF) as u8,
                );

                Ok(Self::A {
                    question,
                    addr,
                    ttl,
                    rd_length,
                })
            }
            QuestionKind::AAAA => {
                let addr = Ipv6Addr::from(buf.read::<u128>()?);
                Ok(Self::AAAA {
                    question,
                    addr,
                    ttl,
                    rd_length,
                })
            }
            QuestionKind::CNAME => Ok(Self::CNAME {
                question,
                ttl,
                rd_length,
                host: buf.read_qname()?,
            }),
            QuestionKind::NS => Ok(Self::NS {
                question,
                ttl,
                rd_length,
                host: buf.read_qname()?,
            }),
            QuestionKind::PTR => Ok(Self::PTR {
                question,
                ttl,
                rd_length,
                host: buf.read_qname()?,
            }),
            QuestionKind::MX => Ok(Self::MX {
                question,
                ttl,
                rd_length,
                preference: buf.read()?,
                exchange: buf.read_qname()?,
            }),
            QuestionKind::SOA => Ok(Self::SOA {
                question,
                ttl,
                rd_length,
                mname: buf.read_qname()?,
                rname: buf.read_qname()?,
                serial: buf.read()?,
                refresh: buf.read()?,
                retry: buf.read()?,
                expire: buf.read()?,
                minimum: buf.read()?,
            }),
        }
    }
}

impl Record {
    pub async fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        match self {
            Record::A {
                question,
                addr,
                ttl,
                rd_length,
            } => {
                question.write(buffer).await?;
                buffer.write(*ttl).await?;
                buffer.write(*rd_length).await?;
                buffer.write(u32::from(*addr)).await?;
            }
            Record::AAAA {
                question,
                addr,
                ttl,
                rd_length,
            } => {
                question.write(buffer).await?;
                buffer.write(*ttl).await?;
                buffer.write(*rd_length).await?;
                buffer.write(u128::from(*addr)).await?;
            }
            Record::CNAME {
                question,
                host,
                ttl,
                rd_length,
            }
            | Record::NS {
                question,
                host,
                ttl,
                rd_length,
            }
            | Record::PTR {
                question,
                host,
                ttl,
                rd_length,
            } => {
                question.write(buffer).await?;
                buffer.write(*ttl).await?;

                let prelength = buffer.pos;
                buffer.write(*rd_length).await?;

                buffer.write_qname(host).await?;
                buffer.set(prelength, (buffer.pos - prelength + 2) as u16)?;
            }
            Record::MX {
                question,
                preference,
                exchange,
                ttl,
                rd_length,
            } => {
                question.write(buffer).await?;
                buffer.write(*ttl).await?;

                let prelength = buffer.pos;
                buffer.write(*rd_length).await?;
                buffer.write(*preference).await?;
                buffer.write_qname(exchange).await?;
                buffer.set(prelength, (buffer.pos - prelength + 2) as u16)?;
            }
            Record::SOA {
                question,
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
                ttl,
                rd_length,
            } => {
                question.write(buffer).await?;
                buffer.write(*ttl).await?;
                let prelength = buffer.pos;
                buffer.write(*rd_length).await?;

                buffer.write_qname(mname).await?;
                buffer.write_qname(rname).await?;
                buffer.write(*serial).await?;
                buffer.write(*refresh).await?;
                buffer.write(*retry).await?;
                buffer.write(*expire).await?;
                buffer.write(*minimum).await?;
                buffer.set(prelength, (buffer.pos - prelength + 2) as u16)?;
            }
            Record::UNKNOWN { .. } => {}
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Packet {
    pub header: Header,
    pub questions: Option<Vec<Question>>,
    pub answers: Option<Vec<Record>>,
    pub authorities: Option<Vec<Record>>,
    pub resources: Option<Vec<Record>>,
}

impl TryFrom<&mut PacketBuffer> for Packet {
    type Error = PacketError;

    #[inline]
    fn try_from(buffer: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let mut packet = Packet {
            header: Header::try_from(buffer as &mut PacketBuffer)?,
            questions: None,
            answers: None,
            authorities: None,
            resources: None,
        };

        if packet.header.query_count > 0 {
            let mut questions = Vec::new();
            for _ in 0..packet.header.query_count {
                questions.push(Question::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.questions = Some(questions);
        }

        if packet.header.answer_count > 0 {
            let mut answers = Vec::new();
            for _ in 0..packet.header.answer_count {
                answers.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.answers = Some(answers);
        }

        if packet.header.authority_count > 0 {
            let mut authorities = Vec::new();
            for _ in 0..packet.header.authority_count {
                authorities.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.authorities = Some(authorities);
        }

        if packet.header.additional_record_count > 0 {
            let mut resources = Vec::new();
            for _ in 0..packet.header.additional_record_count {
                resources.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.resources = Some(resources);
        }
        Ok(packet)
    }
}

impl Packet {
    pub async fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        self.header.write(buffer).await?;
        if let Some(questions) = &self.questions {
            for question in questions.iter() {
                question.write(buffer).await?;
            }
        }

        if let Some(answers) = &self.answers {
            for answer in answers.iter() {
                answer.write(buffer).await?;
            }
        }
        if let Some(authorities) = &self.authorities {
            for authority in authorities.iter() {
                authority.write(buffer).await?;
            }
        }
        if let Some(resources) = &self.resources {
            for resource in resources.iter() {
                resource.write(buffer).await?;
            }
        }
        Ok(())
    }

    pub async fn lookup(
        &self,
        socket: &mut UdpSocket,
        server: &str,
    ) -> Result<Packet, LookupError> {
        let mut req: PacketBuffer = PacketBuffer::default();
        self.write(&mut req).await?;
        socket.send_to(&req.buf[0..req.pos], (server, 53)).await?;

        let mut res = PacketBuffer::default();
        socket.recv_from(&mut res.buf).await?;

        let packet = Packet::try_from(&mut res)?;
        Ok(packet)
    }

    fn get_authority<'a>(&'a self, qname: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.authorities
            .iter()
            .flatten()
            .filter_map(|record| match record {
                Record::NS { question, host, .. } if qname.ends_with(&question.name) => {
                    Some((question.name.as_str(), host.as_str()))
                }
                _ => None,
            })
    }

    fn get_resolved_authority(&self, qname: &str) -> Option<Ipv4Addr> {
        self.get_authority(qname)
            .flat_map(|(_, host)| {
                self.resources
                    .iter()
                    .flatten()
                    .filter_map(move |record| match record {
                        Record::A { question, addr, .. } if question.name == host => Some(*addr),
                        _ => None,
                    })
            })
            .next()
    }

    pub fn get_unresolved_authority<'a>(&'a self, qname: &'a str) -> Option<&'a str> {
        self.get_authority(qname).map(|(_, host)| host).next()
    }

    pub fn get_random_a(&self) -> Option<Ipv4Addr> {
        self.answers
            .iter()
            .flatten()
            .filter_map(|record| match record {
                Record::A { addr, .. } => Some(*addr),
                _ => None,
            })
            .next()
    }
}

// pub async fn recursive_lookup(
//     qname: &str,
//     depth: usize,
//     max_depth: usize,
// ) -> Result<Packet, Box<dyn Error + Send + Sync>> {
//     if depth > max_depth {
//         return Err(LookupError::MaxRecursionDepth(max_depth).into());
//     }

//     let packet = Packet {
//         header: Header {
//             id: 30000,
//             query_count: 1,
//             recursion_desired: true,
//             ..Default::default()
//         },
//         questions: Some(vec![Question {
//             name: qname.to_string(),
//             kind: QuestionKind::A,
//             class: QuestionClass::IN,
//         }]),
//         answers: None,
//         authorities: None,
//         resources: None,
//     };

//     let mut server = NAMED_ROOT[fastrand::usize(..NAMED_ROOT.len())].to_string();
//     let mut socket = UdpSocket::bind(("0.0.0.0", 0)).await?;

//     loop {
//         let res = packet.lookup(&mut socket, &server).await?;
//         if res.answers.is_some() && res.header.response_code == ResponseCode::NoError {
//             return Ok(res);
//         }

//         if let Some(hop) = res.get_resolved_authority(qname) {
//             server = hop.to_string();
//             continue;
//         }

//         let next = match res.get_unresolved_authority(qname) {
//             Some(x) => x,
//             None => return Ok(res),
//         };

//         let recursive_response = recursive_lookup(&next, depth + 1, max_depth).await?;
//         match recursive_response.get_random_a() {
//             Some(next_domain) => server = next_domain.to_string(),
//             _ => return Ok(res),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::{Packet, Record};
    use crate::{
        buffer::{Header, PacketBuffer, ResponseCode},
        question::{Question, QuestionClass, QuestionKind},
    };
    use pretty_assertions::assert_eq;
    use rstest::*;
    use std::{
        net::{Ipv4Addr, Ipv6Addr},
        str::FromStr,
    };
    use tokio::net::UdpSocket;

    #[rstest]
    #[case(
        &[
            0x7d, 0xd8, 
            0x81, 0x80, 0x00, 0x01, 
            0x00, 
            0x01, 
            0x00, 
            0x00, 
            0x00, 0x00, 
            6, b'g', b'o', b'o', b'g', b'l', b'e', 
            3, b'c', b'o', b'm', 0x00, 
            0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 
            0x00, 0x01, 0x2c, 0x00, 0x04, 0xac, 0xd9, 0x14, 0xce,        
        ],
        Packet { 
            header: Header { 
                id: 32216, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: true, 
                recursion_available: true, 
                zero: false, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 1, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::A, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::A { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::A, 
                        class: QuestionClass::IN, 
                    },
                    addr: Ipv4Addr::new(172, 217, 20, 206), 
                    ttl: 300, 
                    rd_length: 4 
                }
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    #[case(
        &[
            0x2E, 0x3E,
            0x81, 0x80, 0x00, 0x01,
            0x00, 0x01,
            0x00, 0x00,
            0x00, 0x00, 
            6, b'g', b'o', b'o', b'g', b'l', b'e', 
            3, b'c', b'o', b'm', 0x00, 
            0x00, 0x1C, 
            0x00, 0x01,
            0xC0, 0x0C,
            0x00, 0x1C,
            0x00, 0x01,
            0x00, 0x00,
            0x01, 0x2C,
            0x00, 0x10,
            0x2A, 0x00,
            0x14, 0x50,
            0x50, 0x40,
            0x07, 0x08,
            0x10, 0x00,
            0x00, 0x00,
            0x00, 0x00,
            0x00, 0x20,
            0x0E
        ],
        Packet { 
            header: Header { 
                id: 11838, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: true, 
                recursion_available: true, 
                zero: false, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 1, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::AAAA, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::AAAA { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::AAAA, 
                        class: QuestionClass::IN, 
                    },
                    addr: Ipv6Addr::from_str("2a00:1450:5040:708:1000::20").unwrap(), 
                    ttl: 300, 
                    rd_length: 16
                }
            ]), 
            authorities: None,
            resources: None 
        }
    )]
    #[case(
        &[
            0x42,0xf4,
            0x81,0x80,0x00,0x01,
            0x00,0x00,0x00,0x01,0x00,0x00,
            6, b'g', b'o', b'o', b'g', b'l', b'e', 
            3, b'c', b'o', b'm', 0x00,
            0x00,0x05,
            0x00,0x01,
            0xc0,0x0c,0x00,0x06,0x00,0x01,
            0x00,0x00,0x00,0x3c,0x00,0x26,
            3,b'n', b's', b'1', 0xc0,
            0x0c,
            9,b'd', b'n', b's', b'-', b'a', b'd', b'm', b'i', b'n' ,0xc0,
            0x0c,0x25,
            0x88,0x0f,0xe9,0x00,0x00,
            0x03,0x84,0x00,0x00,0x03,
            0x84,0x00,0x00,0x07,0x08,
            0x00,0x00,0x00,0x3c
        ],
        Packet { 
            header: Header { 
                id: 17140, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: true, 
                recursion_available: true, 
                zero: false, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 0, 
                authority_count: 1, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::CNAME, 
                    class: QuestionClass::IN, 
                }
            ]), 
            authorities: Some(vec![
                Record::SOA { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::SOA, 
                        class: QuestionClass::IN, 
                    },
                    ttl: 60,
                    rd_length: 38,
                    mname: "ns1.google.com".to_string(),
                    rname: "dns-admin.google.com".to_string(),
                    serial: 629673961,
                    refresh: 900,
                    retry: 900,
                    expire: 1800,
                    minimum: 60,
                }
            ]), 
            answers: None, 
            resources: None 
        }
    )]
    #[case(
        &[
            0x98,0xf3,
            0x81,0x80,0x00,0x01,
            0x00,0x04,0x00,0x00,0x00,0x00,
            6, b'g', b'o', b'o', b'g', b'l', b'e', 
            3, b'c', b'o', b'm', 0x00,
            0, 2,
            0x00,0x01,
            0xc0,0x0c,0x00,0x02,
            0x00,0x01,0x00,0x00,
            0x21,0x25,0x00,0x06,
            3,b'n', b's', b'2',
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,b'n', b's', b'4',
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,b'n', b's', b'1',
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,b'n', b's', b'2',
            0xc0,0x0c
        ],
        Packet { 
            header: Header { 
                id: 39155, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: true, 
                recursion_available: true, 
                zero: false, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 4, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::NS, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::NS { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::NS, 
                        class: QuestionClass::IN, 
                    },
                    host: "ns2.google.com".to_string(),
                    ttl: 8485,
                    rd_length: 6,
                },
                Record::NS { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::NS, 
                        class: QuestionClass::IN, 
                    },
                    host: "ns4.google.com".to_string(),
                    ttl: 8485,
                    rd_length: 6,
                },
                Record::NS { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::NS, 
                        class: QuestionClass::IN, 
                    },
                    host: "ns1.google.com".to_string(),
                    ttl: 8485,
                    rd_length: 6,
                },
                Record::NS { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::NS, 
                        class: QuestionClass::IN, 
                    },
                    host: "ns2.google.com".to_string(),
                    ttl: 8485,
                    rd_length: 6,
                },
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    #[case(
        &[
            0xc2,0x39,
            0x81,0x80,0x00,0x01,
            0x00,0x01,0x00,0x00,0x00,0x00,
            6, b'g', b'o', b'o', b'g', b'l', b'e', 
            3, b'c', b'o', b'm', 0x00,
            0, 15u8,
            0x00,0x01,0xc0,0x0c,0x00,
            0x0f,0x00,0x01,0x00,0x00,
            0x00,0xbf,0x00,0x09,0x00,0x0a,
            4, b's', b'm', b't', b'p',
            0xc0,0x0c
        ],
        Packet { 
            header: Header { 
                id: 49721, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: true, 
                recursion_available: true, 
                zero: false, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 1, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::MX, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::MX { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::MX, 
                        class: QuestionClass::IN, 
                    },
                    ttl: 191,
                    rd_length: 9,
                    preference: 10,
                    exchange: "smtp.google.com".to_string(),
                }
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    fn test_packet_parsing(#[case] input: &[u8], #[case] expected: Packet) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..input.len()].copy_from_slice(input);

        pb.pos = 0;
        let packet = Packet::try_from(&mut pb).unwrap();
        assert_eq!(packet, expected);
    }

    #[rstest]
    #[case(
        Packet { 
            header: Header { 
                id: 16, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: false, 
                recursion_available: false, 
                zero: true, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 1, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::A, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::A { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::A, 
                        class: QuestionClass::IN, 
                    },
                    addr: Ipv4Addr::new(172, 217, 20, 206), 
                    ttl: 300, 
                    rd_length: 4 
                }
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    #[case(
        Packet { 
            header: Header { 
                id: 16, 
                opcode: 0, 
                query: true, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: false, 
                recursion_available: false, 
                zero: true, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 1, 
                answer_count: 1, 
                authority_count: 0, 
                additional_record_count: 0 
            }, 
            questions: Some(vec![
                Question{ 
                    name: "google.com".to_string(), 
                    kind: QuestionKind::NS, 
                    class: QuestionClass::IN, 
                }
            ]), 
            answers: Some(vec![
                Record::NS { 
                    question: Question {
                        name: "google.com".to_string(), 
                        kind: QuestionKind::NS, 
                        class: QuestionClass::IN, 
                    },
                    host: "ns2.google.com".to_string(),
                    ttl: 8485,
                    rd_length: 6,
                },
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    #[tokio::test]
    async fn test_packet_writing(#[case] input: Packet) {
        let mut pb = PacketBuffer::default();
        assert!(input.write(&mut pb).await.is_ok());
        pb.pos = 0;
        assert_eq!(Packet::try_from(&mut pb).unwrap(), input);
    }

    #[tokio::test]
    async fn test_lookup() {
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
        let server = "8.8.8.8";
        let mut socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

        let res = packet.lookup(&mut socket, server).await.unwrap();
        assert!(res.header.answer_count > 0);
    }

    // #[tokio::test]
    // async fn test_recursive_lookup() {
    //     let res = recursive_lookup("google.com", 0, 10).await.unwrap();
    //     assert!(res.header.answer_count > 0);
    // }
}
