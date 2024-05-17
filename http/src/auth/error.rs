use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum AuthorizationError {
    BasicInvalidFormat,
    UnknownScheme,
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "authorization_error")
    }
}

impl Error for AuthorizationError {}
