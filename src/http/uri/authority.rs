use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::http::error::parse::ParseError;

#[derive(Debug, PartialEq)]
pub enum Authority {
    Domain(String),
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
}

impl TryFrom<Authority> for String {
    type Error = ParseError;

    fn try_from(value: Authority) -> Result<Self, Self::Error> {
        match value {
            Authority::Domain(s) => Ok(s),
            Authority::IPv4(ip) => Ok(ip.to_string()),
            Authority::IPv6(ip) => Ok(ip.to_string()),
        }
    }
}

impl FromStr for Authority {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: implement authority parsing
        Ok(Self::Domain(s.to_string()))
    }
}
