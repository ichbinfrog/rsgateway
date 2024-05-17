use std::{error::Error, num::IntErrorKind};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidMethod,
    InvalidStandard,
    MalformedStandardVersion,
    InvalidURI,
    InvalidAuthority {
        reason: &'static str,
    },

    InvalidInteger {
        reason: String,
        subject: &'static str,
    },
    MissingContentLengthHeader,
    MalformedQuery,

    HexInvalidStringLength {
        index: usize,
    },
    InvalidPercentEncoding {
        index: usize,
        kind: IntErrorKind,
    },

    InvalidMimeType {
        reason: &'static str,
    },
    UnknownStatusCode,

    ContentTooLarge {
        subject: String,
    },
    HeaderNotFound,
    HeaderStructuredGetNotImplemented,

    InvalidUserAgent {
        reason: String,
    },

    NotImplemented,

    AuthorizationMissingRequiredParam {
        subject: &'static str,
    },
    AuthorizationBasicInvalidFormat,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for ParseError {}
