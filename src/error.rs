use std::{error::Error, num::{IntErrorKind, ParseIntError}};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidMethod,
    InvalidStandard,
    MalformedStandardVersion,
    InvalidURI,
    MalformedQuery,

    HexInvalidStringLength,
    HexParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(value: ParseIntError) -> Self {
        ParseError::HexParseIntError(value)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl Error for ParseError {}
