use super::packet::{PacketBuffer, PacketError};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum QuestionKind {
    A = 1,
    NS = 2,
    CNAME = 5,
    SOA = 6,

    PTR = 12,
    MX = 15,
    AAAA = 28,
}

impl TryFrom<u16> for QuestionKind {
    type Error = PacketError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(QuestionKind::A),
            2 => Ok(QuestionKind::NS),
            5 => Ok(QuestionKind::CNAME),
            6 => Ok(QuestionKind::SOA),
            12 => Ok(QuestionKind::PTR),
            15 => Ok(QuestionKind::MX),
            28 => Ok(QuestionKind::AAAA),
            _ => Err(PacketError::NotImplemented {
                reason: "unknown_qtype".to_string(),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum QuestionClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,
}

impl TryFrom<u16> for QuestionClass {
    type Error = PacketError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(QuestionClass::IN),
            2 => Ok(QuestionClass::CS),
            3 => Ok(QuestionClass::CH),
            4 => Ok(QuestionClass::HS),
            _ => Err(PacketError::NotImplemented {
                reason: "unknown_qclass".to_string(),
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Question {
    pub name: String,
    pub kind: QuestionKind,
    pub class: QuestionClass,
}

impl TryFrom<&mut PacketBuffer> for Question {
    type Error = PacketError;

    fn try_from(buffer: &mut PacketBuffer) -> Result<Self, Self::Error> {
        Ok(Self {
            name: buffer.read_qname()?,
            kind: QuestionKind::try_from(buffer.read::<u16>()?)?,
            class: QuestionClass::try_from(buffer.read::<u16>()?)?,
        })
    }
}

impl Question {
    pub fn write(&self, buffer: &mut PacketBuffer) -> Result<(), PacketError> {
        buffer.write_qname(self.name.as_str())?;
        buffer.write(self.kind as u16)?;
        buffer.write(self.class as u16)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        &[
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8,
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0,
            0, 01,
            0, 01,
        ],
        Question { 
            name: "google.com".to_string(), 
            kind: QuestionKind::A, 
            class: QuestionClass::IN, 
        },
    )]
    #[case(
        &[
            6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8,
            3, 'c' as u8, 'o' as u8, 'm' as u8, 0,
            0, 05,
            0, 01,
        ],
        Question { 
            name: "google.com".to_string(), 
            kind: QuestionKind::CNAME, 
            class: QuestionClass::IN, 
        },
    )]
    fn test_question_parse(#[case] input: &[u8], #[case] expected: Question) {
        let mut pb = PacketBuffer::default();
        pb.buf[0..input.len()].copy_from_slice(input);
        pb.pos = 0;
        let res = Question::try_from(&mut pb).unwrap();
        assert_eq!(res, expected);
    }
}
