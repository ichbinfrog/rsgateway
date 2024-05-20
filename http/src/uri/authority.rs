use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use crate::error::frame::FrameError;

#[derive(Debug, PartialEq, Clone)]
pub enum Authority {
    Domain { host: String, port: usize },
    IPv4 { ip: Ipv4Addr, port: usize },
    IPv6 { ip: Ipv6Addr, port: usize },
    Undefined,
}

impl TryFrom<Authority> for String {
    type Error = FrameError;

    fn try_from(value: Authority) -> Result<Self, Self::Error> {
        match value {
            Authority::Domain { host, port } => {
                let mut res = host;
                res.push(':');
                res.push_str(port.to_string().as_str());
                Ok(res)
            }
            Authority::IPv4 { ip, port } => {
                let mut res = ip.to_string();
                res.push(':');
                res.push_str(port.to_string().as_str());
                Ok(res)
            }
            Authority::IPv6 { ip, port } => {
                let mut res = "[".to_string();
                res.push_str(&ip.to_string());
                res.push(']');
                res.push(':');
                res.push_str(port.to_string().as_str());
                Ok(res)
            }
            Authority::Undefined => Err(FrameError::Invalid {
                subject: "authority",
                reason: "undefined authority",
            }),
        }
    }
}

impl FromStr for Authority {
    type Err = FrameError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut s = input;
        let mut port: usize = 80;

        //  IPv6address = [ ipv6 ](:port)?
        //  IPv4address = [ dec ].[ dec ].[ dec ].[ dec ](:port)?
        //  Host        = [ host ](:port)?
        if let Some((left, right)) = s.rsplit_once(':') {
            if !right.contains(']') {
                s = left;
                port = str::parse(right)?;
            }
        }

        // '[' => false, ']' => false, '[]' => true, '' => true
        if s.starts_with('[') ^ s.ends_with(']') {
            return Err(FrameError::Invalid {
                subject: "authority",
                reason: "ipv6 host is missing either closing or opening brackets",
            });
        }

        if s.starts_with('[') && s.ends_with(']') {
            return Ok(Self::IPv6 {
                ip: Ipv6Addr::from_str(&s[1..s.len() - 1])?,
                port,
            });
        }

        let split: Vec<&str> = s.split('.').collect();
        if split.len() == 4 {
            if let Ok(ip) = Ipv4Addr::from_str(s) {
                return Ok(Self::IPv4 { ip, port });
            }
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
        "192.168.0.1", Authority::IPv4 { ip: Ipv4Addr::new(192, 168, 0, 1), port: 80 }
    )]
    #[case(
        "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:9000",
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: 9000}
    )]
    #[case(
        "[2001:0db8:85a3:0000:0000:8a2e:0370:7334]", 
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: 80}
    )]
    #[case(
        "[::1]", Authority::IPv6{ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port: 80}
    )]
    #[case(
        "[::1]:8000", Authority::IPv6{ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port: 8000}
    )]
    #[case(
        "localhost", Authority::Domain{host: "localhost".to_string(), port: 80}
    )]
    #[case(
        "localhost:10000", Authority::Domain{host: "localhost".to_string(), port: 10000}
    )]
    fn test_authority_parse(#[case] input: &str, #[case] expected: Authority) {
        assert_eq!(Authority::from_str(input).unwrap(), expected)
    }

    #[rstest]
    #[case("[")]
    #[case("]")]
    fn test_authority_parse_error(#[case] input: &str) {
        let res = Authority::from_str(input);
        println!("{:?}", res);
        assert!(res.is_err());
    }

    #[rstest]
    #[rstest]
    #[case(
        Authority::IPv4 { ip: Ipv4Addr::new(192, 168, 0, 1), port: 80 }, "192.168.0.1:80",
    )]
    #[case(
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: 9000},
        "[2001:db8:85a3::8a2e:370:7334]:9000"
    )]
    #[case(
        Authority::IPv6{ip: Ipv6Addr::new(0x2001, 0x0db8, 0x85a3, 0x0000, 0x0000, 0x8a2e, 0x0370, 0x7334), port: 80},
        "[2001:db8:85a3::8a2e:370:7334]:80"
    )]
    #[case(
        Authority::IPv6{ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port: 80},"[::1]:80"
    )]
    #[case(
        Authority::IPv6{ip: Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port: 8000},"[::1]:8000"
    )]
    #[case(
        Authority::Domain{host: "localhost".to_string(), port: 80}, "localhost:80"
    )]
    #[case(
        Authority::Domain{host: "localhost".to_string(), port: 10000},"localhost:10000"
    )]
    fn test_authority_tostring(#[case] input: Authority, #[case] expected: &str) {
        assert_eq!(String::try_from(input).unwrap(), expected)
    }

    #[rstest]
    #[case(Authority::Undefined)]
    fn test_authority_tostring_error(#[case] input: Authority) {
        assert!(String::try_from(input).is_err());
    }
}
