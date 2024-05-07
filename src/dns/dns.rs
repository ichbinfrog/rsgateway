use std::{
    error::Error,
    net::{Ipv4Addr, Ipv6Addr, UdpSocket},
};

use super::{
    packet::{Header, PacketBuffer, PacketError},
    question::{Question, QuestionKind},
};

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
                    ((raw >> 0) & 0xFF) as u8,
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
    pub fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        match self {
            Record::A {
                question,
                addr,
                ttl,
                rd_length,
            } => {
                question.write(buffer)?;
                buffer.write(*ttl)?;
                buffer.write(*rd_length)?;
                buffer.write(u32::from(*addr))?;
            }
            Record::AAAA {
                question,
                addr,
                ttl,
                rd_length,
            } => {
                question.write(buffer)?;
                buffer.write(*ttl)?;
                buffer.write(*rd_length)?;
                buffer.write(u128::from(*addr))?;
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
                question.write(buffer)?;
                buffer.write(*ttl)?;

                let prelength = buffer.pos;
                buffer.write(*rd_length)?;

                buffer.write_qname(host)?;
                buffer.set(prelength, (buffer.pos - prelength + 2) as u16)?;
            }
            Record::MX {
                question,
                preference,
                exchange,
                ttl,
                rd_length,
            } => {
                question.write(buffer)?;
                buffer.write(*ttl)?;

                let prelength = buffer.pos;
                buffer.write(*rd_length)?;
                buffer.write(*preference)?;
                buffer.write_qname(&exchange)?;
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
                question.write(buffer)?;
                buffer.write(*ttl)?;
                let prelength = buffer.pos;
                buffer.write(*rd_length)?;

                buffer.write_qname(&mname)?;
                buffer.write_qname(&rname)?;
                buffer.write(*serial)?;
                buffer.write(*refresh)?;
                buffer.write(*retry)?;
                buffer.write(*expire)?;
                buffer.write(*minimum)?;
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
    pub fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        self.header.write(buffer)?;
        if let Some(questions) = &self.questions {
            for question in questions.iter() {
                question.write(buffer)?;
            }
        }

        if let Some(answers) = &self.answers {
            for answer in answers.iter() {
                answer.write(buffer)?;
            }
        }
        if let Some(authorities) = &self.authorities {
            for authority in authorities.iter() {
                authority.write(buffer)?;
            }
        }
        if let Some(resources) = &self.resources {
            for resource in resources.iter() {
                resource.write(buffer)?;
            }
        }
        Ok(())
    }

    pub fn lookup(&self, socket: UdpSocket, server: &str) -> Result<Packet, Box<dyn Error>> {
        let mut req: PacketBuffer = PacketBuffer::default();
        self.write(&mut req)?;
        socket.send_to(&req.buf[0..req.pos], server)?;

        let mut res = PacketBuffer::default();
        socket.recv_from(&mut res.buf)?;

        let packet = Packet::try_from(&mut res)?;
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::{Packet, Record};
    use crate::dns::{
        packet::{Header, PacketBuffer, ResponseCode},
        question::{Question, QuestionClass, QuestionKind},
    };
    use pretty_assertions::assert_eq;
    use rstest::*;
    use std::{
        net::{Ipv4Addr, Ipv6Addr, UdpSocket},
        str::FromStr,
    };

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
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00, 
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
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00, 
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
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00,
            0x00,0x05,
            0x00,0x01,
            0xc0,0x0c,0x00,0x06,0x00,0x01,
            0x00,0x00,0x00,0x3c,0x00,0x26,
            3,'n' as u8, 's' as u8, '1' as u8, 0xc0,
            0x0c,
            9,'d' as u8, 'n' as u8, 's' as u8, '-' as u8, 'a' as u8, 'd' as u8, 'm' as u8, 'i' as u8, 'n' as u8 ,0xc0,
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
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00,
            0, 2 as u8,
            0x00,0x01,
            0xc0,0x0c,0x00,0x02,
            0x00,0x01,0x00,0x00,
            0x21,0x25,0x00,0x06,
            3,'n' as u8, 's' as u8, '2' as u8,
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,'n' as u8, 's' as u8, '4' as u8,
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,'n' as u8, 's' as u8, '1' as u8,
            0xc0,0x0c,0xc0,0x0c,
            0x00,0x02,0x00,0x01,
            0x00,0x00,0x21,0x25,
            0x00,0x06,
            0x03,'n' as u8, 's' as u8, '2' as u8,
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
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00,
            0, 15 as u8,
            0x00,0x01,0xc0,0x0c,0x00,
            0x0f,0x00,0x01,0x00,0x00,
            0x00,0xbf,0x00,0x09,0x00,0x0a,
            4, 's' as u8, 'm' as u8, 't' as u8, 'p' as u8,
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
    fn test_packet_writing(#[case] input: Packet) {
        let mut pb = PacketBuffer::default();
        assert!(input.write(&mut pb).is_ok());
        pb.pos = 0;
        assert_eq!(Packet::try_from(&mut pb).unwrap(), input);
    }

    #[test]
    fn test_lookup() {
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
        let server = "8.8.8.8:53";
        let socket = UdpSocket::bind(("0.0.0.0", 43210)).unwrap();

        let res = packet.lookup(socket, server).unwrap();
        println!("{:?}", res);
    }
}
