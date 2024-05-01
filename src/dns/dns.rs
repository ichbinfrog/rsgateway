use std::net::Ipv4Addr;

use super::{
    packet::{Header, PacketBuffer, PacketError},
    question::{Question, QuestionKind},
};

#[derive(Debug, PartialEq)]
pub enum Record {
    UNKNOWN {
        domain: String,
        ttl: u32,
        rd_length: u16,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
        rd_length: u16,
    },
    CNAME,
    NS,
}

impl TryFrom<&mut PacketBuffer> for Record {
    type Error = PacketError;
    fn try_from(buf: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let question = Question::try_from(buf as &mut PacketBuffer)?;
        let ttl = buf.read_u32()?;
        let rd_length = buf.read_u16()?;

        match question.kind {
            QuestionKind::A => {
                let raw = buf.read_u32()?;

                let addr = Ipv4Addr::new(
                    ((raw >> 24) & 0xFF) as u8,
                    ((raw >> 16) & 0xFF) as u8,
                    ((raw >> 8) & 0xFF) as u8,
                    ((raw >> 0) & 0xFF) as u8,
                );

                Ok(Self::A {
                    domain: question.name,
                    addr,
                    ttl,
                    rd_length,
                })
            }
            QuestionKind::NotImplemented => Ok(Self::UNKNOWN {
                domain: question.name,
                ttl,
                rd_length,
            }),
        }
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

    fn try_from(buffer: &mut PacketBuffer) -> Result<Self, Self::Error> {
        let mut packet = Packet {
            header: Header::default(),
            questions: None,
            answers: None, 
            authorities: None,
            resources: None,
        };
        let header = Header::try_from(buffer as &mut PacketBuffer)?;

        if header.query_count > 0 {
            let mut questions= Vec::new();
            for _ in 0..header.query_count {
                questions.push(Question::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.questions = Some(questions);
        }

        if header.answer_count > 0 {
            let mut answers = Vec::new();
            for _ in 0..header.answer_count {
                answers.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.answers = Some(answers);
        }

        if header.authority_count > 0 {
            let mut authorities = Vec::new();
            for _ in 0..header.authority_count {
                authorities.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.authorities = Some(authorities);
        }

        if header.additional_record_count > 0 {
            let mut resources = Vec::new();
            for _ in 0..header.additional_record_count {
                resources.push(Record::try_from(buffer as &mut PacketBuffer)?)
            }
            packet.resources = Some(resources);
        }
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use crate::dns::{packet::{Header, PacketBuffer, ResponseCode}, question::{Question, QuestionClass, QuestionKind}};
    use super::{Packet, Record};
    use rstest::*;


    #[rstest]
    #[case(
        &[
            0x7d, 0xd8, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
            3, 'c' as u8, 'o' as u8, 'm' as u8,
           0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 
           0x00, 0x01, 0x2c, 0x00, 0x04, 0xac, 0xd9, 0x14, 0xce,        
        ],
        Packet { 
            header: Header { 
                id: 0, 
                opcode: 0, 
                query: false, 
                authoritative_answer: false, 
                truncated_message: false, 
                recursion_desired: false, 
                recursion_available: false, 
                zero: true, 
                authed_data: false, 
                checking_disabled: false, 
                response_code: ResponseCode::NoError, 
                query_count: 0, 
                answer_count: 0, 
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
                    domain: "google.com".to_string(), 
                    addr: Ipv4Addr::new(172, 217, 20, 206), 
                    ttl: 300, 
                    rd_length: 4 
                }
            ]), 
            authorities: None, 
            resources: None 
        }
    )]
    fn test_packet_parsing(#[case] input: &[u8], #[case] expected: Packet) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..input.len()].copy_from_slice(input);

        let packet = Packet::try_from(&mut pb).unwrap();
        assert_eq!(packet, expected);
    }
}