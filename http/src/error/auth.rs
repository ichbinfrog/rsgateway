use std::error::Error;

use encoding::error::EncodingError;

#[derive(Debug, PartialEq)]
pub enum AuthorizationError {
    InvalidFormat { reason: &'static str },
    EncodingError(EncodingError),
    UnknownScheme,
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "authorization_error")
    }
}

impl Error for AuthorizationError {}
impl From<EncodingError> for AuthorizationError {
    fn from(_src: EncodingError) -> Self {
        Self::EncodingError(_src)
    }
}

#[derive(Debug, PartialEq)]
pub enum AuthenticationError {
    RequiredParam { subject: &'static str },
}

impl std::fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "authentication_error")
    }
}

impl Error for AuthenticationError {}
