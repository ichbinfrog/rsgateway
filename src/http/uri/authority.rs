use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::http::error::parse::ParseError;

#[derive(Debug, PartialEq)]
pub enum Authority {
    Domain { host: String, port: Option<usize> },
    IPv4 { ip: Ipv4Addr, port: Option<usize> },
    IPv6 { ip: Ipv6Addr, port: Option<usize> },
}

impl TryFrom<Authority> for String {
    type Error = ParseError;

    fn try_from(value: Authority) -> Result<Self, Self::Error> {
        match value {
            Authority::Domain { host, port } => {
                let mut res = host;
                if let Some(port) = port {
                    res.push_str(port.to_string().as_str());
                }
                Ok(res)
            }
            Authority::IPv4 { ip, port } => {
                let mut res = ip.to_string();
                if let Some(port) = port {
                    res.push_str(port.to_string().as_str());
                }
                Ok(res)
            }
            Authority::IPv6 { ip, port } => {
                let mut res = ip.to_string();
                if let Some(port) = port {
                    res.push_str(port.to_string().as_str());
                }
                Ok(res)
            }
        }
    }
}

impl FromStr for Authority {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut s = input;
        let mut port: Option<usize> = None;

        if let Some(last) = s.chars().last() {
            if last != ']' {
                match s.rsplit_once(':') {
                    Some((authority, right)) => {
                        if right != "" {
                            match usize::from_str_radix(right, 10) {
                                Ok(n) => port = Some(n),
                                Err(e) => {
                                    return Err(ParseError::InvalidIPV6Authority {
                                        reason: e.to_string(),
                                    }
                                    .into())
                                }
                            }
                        }
                        s = authority;
                    }
                    None => {}
                }
            }
        }

        if let Some(first) = s.chars().nth(0) {
            if first == '[' {
                if let Some(last) = s.chars().last() {
                    if last == ']' && s.len() > 2 {
                        match Ipv6Addr::from_str(&s[1..s.len() - 1]) {
                            Ok(ip) => return Ok(Self::IPv6 { ip, port }),
                            Err(e) => {
                                return Err(ParseError::InvalidIPV6Authority {
                                    reason: e.to_string(),
                                }
                                .into())
                            }
                        }
                    }
                }
                return Err(ParseError::InvalidIPV6Authority {
                    reason: "ipv6 host should end with ]".to_string(),
                }
                .into());
            }
        }

        match Ipv4Addr::from_str(s) {
            Ok(ip) => return Ok(Self::IPv4 { ip, port }),
            _ => {}
        }

        Ok(Self::Domain {
            host: s.to_string(),
            port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        "192.168.0.1", Authority::IPv4 { ip: Ipv4Addr::new(192, 168, 0, 1), port: None }
    )]
    #[case(
        "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:9000",
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: Some(9000)}
    )]
    #[case(
        "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]", 
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: None}
    )]
    #[case(
        "[::1]", Authority::IPv6{ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port: None}
    )]
    #[case(
        "localhost", Authority::Domain{host: "localhost".to_string(), port: None}
    )]
    #[case(
        "localhost:10000", Authority::Domain{host: "localhost".to_string(), port: Some(10000)}
    )]
    fn test_authority_parse(#[case] input: &str, #[case] expected: Authority) {
        assert_eq!(Authority::from_str(input).unwrap(), expected)
    }
}
