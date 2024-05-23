use std::{
    fmt::Display,
    num::{ParseFloatError, ParseIntError},
};

use crate::parser::Token;

#[derive(Debug)]
pub struct ParserError {
    pub token: Option<Token>,
    pub reason: String,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("parse_error")
            .field("token", &self.token)
            .field("reason", &self.reason)
            .finish()
    }
}
impl std::error::Error for ParserError {}
impl From<ParseIntError> for ParserError {
    fn from(src: ParseIntError) -> Self {
        ParserError {
            token: None,
            reason: src.to_string(),
        }
    }
}
impl From<ParseFloatError> for ParserError {
    fn from(src: ParseFloatError) -> Self {
        ParserError {
            token: None,
            reason: src.to_string(),
        }
    }
}
impl From<std::io::Error> for ParserError {
    fn from(src: std::io::Error) -> Self {
        ParserError {
            token: None,
            reason: src.to_string(),
        }
    }
}
