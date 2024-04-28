use crate::error::ParseError;
use crate::version::Version;

use std::error::Error;
use std::io::BufRead;
use std::str::FromStr;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Standard {
    name: String,
    version: Version,
}

impl FromStr for Standard {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split('/').collect();
        match split.len() {
            2 => {
                let (name, version) = (split[0], split[1]);
                return Ok(Standard {
                    name: name.to_string(),
                    version: Version::from_str(version)?,
                });
            }
            _ => {}
        }
        Err(ParseError::InvalidStandard.into())
    }
}

#[derive(Debug)]
pub struct Request {
    method: Method,
    standard: Standard,
    // uri: String,
}

pub fn parse<R: BufRead>(reader: R) -> Result<Request, Box<dyn Error>> {
    let mut request = Request {
        method: Method::UNDEFINED,
        standard: Standard {
            name: "".to_string(),
            version: Version {
                major: 0,
                minor: None,
                patch: None,
            },
        },
    };

    for (i, line) in reader.lines().enumerate() {
        let mut line = line?;
        line.push(' ');

        if i == 0 {
            let mut acc = String::with_capacity(line.len());
            let mut j = 0;
            for ch in line.chars() {
                if ch.is_whitespace() {
                    match j {
                        0 => request.method = Method::from_str(&acc)?,
                        1 => {}
                        2 => request.standard = Standard::from_str(&acc)?,
                        _ => {}
                    }
                    j += 1;
                    acc.clear()
                } else {
                    acc.push(ch)
                }
            }
        }
    }

    Ok(request)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_parse() {
        let first = BufReader::new("GET / HTTP/1.1".as_bytes());
        println!("{:?}", parse(first));
    }
}
