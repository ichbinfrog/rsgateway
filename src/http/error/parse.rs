use std::{error::Error, num::IntErrorKind};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidMethod,
    InvalidStandard,
    MalformedStandardVersion,
    InvalidURI,
    MalformedQuery,

    HexInvalidStringLength { index: usize },
    HexParseIntError { index: usize, kind: IntErrorKind },

    InvalidMimeType { reason: String },

    ContentTooLarge { subject: String },
    HeaderNotFound,
    HeaderStructuredGetNotImplemented,

    InvalidUserAgent { reason: String },

    NotImplemented,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for ParseError {}
