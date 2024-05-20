use std::{error::Error, num::IntErrorKind};

#[derive(Debug, PartialEq)]
pub enum EncodingError {
    Base64UnknownCharacter { character: char },
    PercentHexInvalidStringLength { index: usize },
    PercentInvalidInteger { index: usize, kind: IntErrorKind },
}

impl std::fmt::Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "encoding_error")
    }
}

impl Error for EncodingError {}
