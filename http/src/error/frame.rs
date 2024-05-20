use std::{error::Error, net::AddrParseError, num::ParseIntError};

use dns::error::LookupError;
use encoding::error::EncodingError;

use super::auth::AuthorizationError;

#[derive(Debug)]
pub enum FrameError {
    Invalid {
        reason: &'static str,
        subject: &'static str,
    },
    NotImplemented {
        subject: String,
    },
    RequiredParam {
        subject: &'static str,
    },
    ConversionError(String),
    AuthorizationError(AuthorizationError),
    EncodingError(EncodingError),
    ContentTooLarge {
        subject: String,
    },
    HeaderNotFound,
    IOError(std::io::Error),
    LookupError(LookupError),
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for FrameError {}

impl From<ParseIntError> for FrameError {
    fn from(src: ParseIntError) -> Self {
        Self::ConversionError(src.to_string())
    }
}

impl From<EncodingError> for FrameError {
    fn from(src: EncodingError) -> Self {
        Self::EncodingError(src)
    }
}

impl From<AddrParseError> for FrameError {
    fn from(src: AddrParseError) -> Self {
        Self::ConversionError(src.to_string())
    }
}

impl From<AuthorizationError> for FrameError {
    fn from(src: AuthorizationError) -> Self {
        Self::AuthorizationError(src)
    }
}

impl From<std::io::Error> for FrameError {
    fn from(src: std::io::Error) -> Self {
        Self::IOError(src)
    }
}

impl From<LookupError> for FrameError {
    fn from(src: LookupError) -> Self {
        Self::LookupError(src)
    }
}
