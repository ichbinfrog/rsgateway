use crate::http::error::parse::ParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,

    UNDEFINED,
}

impl FromStr for Method {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(ParseError::InvalidMethod),
        }
    }
}

impl TryFrom<Method> for String {
    type Error = ParseError;

    fn try_from(method: Method) -> Result<Self, Self::Error> {
        match method {
            Method::GET => Ok("GET".to_string()),
            Method::HEAD => Ok("HEAD".to_string()),
            Method::POST => Ok("POST".to_string()),
            Method::PUT => Ok("PUT".to_string()),
            Method::DELETE => Ok("DELETE".to_string()),
            Method::CONNECT => Ok("CONNECT".to_string()),
            Method::OPTIONS => Ok("OPTIONS".to_string()),
            Method::TRACE => Ok("TRACE".to_string()),
            _ => Err(ParseError::InvalidMethod),
        }
    }
}
