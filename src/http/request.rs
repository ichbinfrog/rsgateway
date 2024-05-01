use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;

use crate::http::error::parse::ParseError;
use crate::http::method::Method;
use crate::http::version::Version;

use std::error::Error;
use std::str::FromStr;

use super::header::{HeaderKind, HeaderMap};
use super::uri::path::Path;

pub struct Builder {
    inner: Result<Parts, ParseError>,
}

pub struct Parts {
    pub method: Method,
    pub standard: Standard,
    pub headers: HeaderMap,
}

#[derive(Debug, PartialEq)]
pub struct Standard {
    name: String,
    version: Version,
}

impl Default for Standard {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            version: Version::default(),
        }
    }
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

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: Method,
    pub standard: Standard,
    pub headers: HeaderMap,
    pub path: Path,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            method: Method::UNDEFINED,
            standard: Standard::default(),
            path: Path::default(),
            headers: HeaderMap::default(),
        }
    }
}

impl FromStr for Request {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request: Request = Default::default();

        for (i, line) in s.lines().enumerate() {
            let mut line = line.to_string();
            line.push(' ');

            match i {
                0 => {
                    let mut acc = String::with_capacity(line.len());
                    let mut j = 0;
                    for ch in line.chars() {
                        if ch.is_whitespace() {
                            match j {
                                0 => request.method = Method::from_str(&acc)?,
                                1 => request.path = Path::from_str(&acc)?,
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
                _ => {
                    let _ = request.headers.parse(&line);
                }
            }
        }

        Ok(request)
    }
}

impl Request {
    pub async fn body(
        &self,
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let header = self.headers.get("content-length")?;
        match header {
            HeaderKind::ContentLength(n) => {
                let mut buf: Vec<u8> = Vec::with_capacity(n);
                buf.resize(n, 0);

                reader.read_exact(&mut buf).await?;
                Ok(buf)
            }
            _ => {
                // TODO: implement chunk encoding
                Err(ParseError::NotImplemented.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::uri::path::Path;
    use std::collections::HashMap;

    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        vec![
            "GET / HTTP/1.1",
            "Host: localhost:9090",
        ],
        Request { 
            method: Method::GET,
            standard: Standard { 
                name: "HTTP".to_string(), 
                version: Version {
                    major: 1, 
                    minor: Some(1), 
                    patch: None 
                },
            },
            path: Path {
                raw_path: "/".to_string(),
                ..Default::default()
            },
            headers: HeaderMap {
                raw: HashMap::from([
                    ("host".to_string(), "localhost:9090".to_string()),
                ]),
                size: 18,
                ..Default::default()
            },
        }
    )]
    fn test_parse_request(#[case] input: Vec<&str>, #[case] expected: Request) {
        let req = Request::from_str(input.join("\n").as_str()).unwrap();
        assert_eq!(req, expected);
        println!("{:?}", req);
    }
}
