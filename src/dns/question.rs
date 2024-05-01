use super::packet::{PacketBuffer, PacketError};

#[derive(Debug, PartialEq)]
pub enum QuestionKind {
    A = 1,
    // NS = 2,
    NotImplemented,
}

impl From<u16> for QuestionKind {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionKind::A,
            // 2 => QuestionKind::NS,
            _ => QuestionKind::NotImplemented,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QuestionClass {
    IN = 1,
    CS = 2,
    CH = 3,
    HS = 4,

    NotImplemented,
}

impl From<u16> for QuestionClass {
    fn from(value: u16) -> Self {
        match value {
            1 => QuestionClass::IN,
            2 => QuestionClass::CS,
            3 => QuestionClass::CH,
            4 => QuestionClass::HS,
            _ => QuestionClass::NotImplemented,
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
            kind: QuestionKind::from(buffer.read_u16()?),
            class: QuestionClass::from(buffer.read_u16()?),
        })
    }
}
