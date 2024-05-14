use crate::http::error::parse::ParseError;
use crate::http::header::HeaderKind;
use crate::http::method::Method;
use crate::http::version::Version;

use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::str::FromStr;

use super::header::HeaderMap;
use super::response::Response;
use super::traits::TryClone;
use super::uri::path::Path;
use super::uri::url::Url;

const MAX_REQUEST_LINE_SIZE: usize = 8096 * 4;

#[derive(Debug, PartialEq, Clone)]
pub struct Parts {
    pub method: Method,
    pub url: Url,
    pub standard: Standard,

    pub headers: HeaderMap,
}

impl TryFrom<Parts> for String {
    type Error = Box<dyn Error>;

    fn try_from(p: Parts) -> Result<Self, Self::Error> {
        let mut res = String::new();
        res.push_str(&String::try_from(p.method)?);
        res.push(' ');
        res.push_str(&String::try_from(p.url.path)?);
        res.push(' ');
        res.push_str(&String::try_from(p.standard)?);
        res.push_str("\r\n");
        res.push_str(&String::try_from(p.headers)?);
        res.push_str("\r\n");

        Ok(res)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Standard {
    pub name: String,
    pub version: Version,
}

impl Default for Standard {
    fn default() -> Self {
        Self {
            name: "HTTP".to_string(),
            version: Version {
                major: 1,
                minor: Some(1),
                patch: None,
            },
        }
    }
}

impl TryFrom<Standard> for String {
    type Error = ParseError;

    fn try_from(standard: Standard) -> Result<Self, Self::Error> {
        let mut res = standard.name;
        res.push('/');
        res.push_str(&String::try_from(standard.version)?);
        Ok(res)
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

#[derive(Debug, Clone)]
pub struct Request<T> {
    pub parts: Parts,
    pub hasbody: bool,

    pub body: Option<Vec<u8>>,
    pub stream: Option<T>,
}

impl<T> PartialEq for Request<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.parts == other.parts && self.body == other.body;
    }
}

impl<T> Default for Request<T> {
    fn default() -> Self {
        Self {
            parts: Parts {
                method: Method::UNDEFINED,
                standard: Standard::default(),
                url: Url::default(),
                headers: HeaderMap::default(),
            },
            body: None,
            stream: None,
            hasbody: false,
        }
    }
}

impl<T: Read + TryClone<T>> Request<T> {
    pub fn write(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let req = String::try_from(self.parts)?;
        stream.write(req.as_bytes())?;

        Ok(())
    }

    pub fn call(self, stream: &mut TcpStream) -> Result<Response<TcpStream>, Box<dyn Error>> {
        let req = String::try_from(self.parts)?;
        stream.write(req.as_bytes())?;

        let resp = Response::parse(stream)?;
        Ok(resp)
    }

    pub fn parse(stream: &mut T) -> Result<(Self, BufReader<T>), Box<dyn Error>> {
        let mut buffer = BufReader::new(stream.clone()?);
        let mut request = Request::default();
        let mut line = String::with_capacity(MAX_REQUEST_LINE_SIZE);
        let mut state: u8 = 0;

        loop {
            // if state == 2 {
            //     match request.parts.method {
            //         Method::POST => {
            //             // Should only be reachable if method == POST
            //             match request.parts.headers.get("content-length")? {
            //                 HeaderKind::ContentLength(n) => {
            //                     println!("reading {:?}", n);

            //                     let mut buf = Vec::<u8>::with_capacity(n);
            //                     stream.read_exact(&mut buf)?;
            //                     println!("{:?}", buf);
            //                 }
            //                 _ => {
            //                     println!("holla");
            //                     break
            //                 }
            //             }

            //             // request.body = Some(stream.read_lin);
            //         }
            //         _ => {
            //             break
            //         }
            //     }
            // }
            match buffer.read_line(&mut line) {
                Ok(0) => {
                    break;
                }
                Ok(_n) => match state {
                    0 => {
                        let mut acc = String::with_capacity(line.len());
                        let mut j = 0;
                        for ch in line.chars() {
                            if ch.is_whitespace() {
                                match j {
                                    0 => request.parts.method = Method::from_str(&acc)?,
                                    1 => request.parts.url.path = Path::from_str(&acc)?,
                                    2 => request.parts.standard = Standard::from_str(&acc)?,
                                    _ => {}
                                }
                                j += 1;
                                acc.clear()
                            } else {
                                acc.push(ch)
                            }
                        }
                        state += 1;
                        line.clear();
                    }
                    1 => {
                        if line == "\r\n" {
                            line.clear();

                            if request.parts.method == Method::POST {
                                request.hasbody = true;
                            }
                            break;
                        }
                        let _ = request.parts.headers.parse(&line);
                        line.clear();
                    }
                    _ => {
                        break;
                    }
                },
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok((request, buffer))
    }

    pub fn read_body(&mut self, buffer: &mut BufReader<T>) -> Result<usize, Box<dyn Error>> {
        if self.hasbody {
            match self.parts.headers.get("content-length") {
                Ok(value) => match value {
                    HeaderKind::ContentLength(n) => {
                        let mut body = Vec::new();
                        body.resize(n, 0u8);
                        buffer.read_exact(&mut body)?;
                        self.body = Some(body);
                        return Ok(n);
                    }
                    _ => return Err(ParseError::MissingContentLengthHeader.into()),
                },
                Err(e) => return Err(e.into()),
            }
        }
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::uri::authority::Authority;
    use crate::http::uri::path::Path;
    use std::{collections::HashMap, io::Cursor, net::TcpStream};

    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        vec![
            "GET / HTTP/1.1",
            "Host: localhost:9090",
        ],
        Request { 
            parts: Parts {
                method: Method::GET,
                standard: Standard { 
                    name: "HTTP".to_string(), 
                    version: Version {
                        major: 1, 
                        minor: Some(1), 
                        patch: None 
                    },
                },
                url: Url {
                    scheme: "http".to_string(),
                    authority: Authority::Domain { host: "localhost".to_string(), port: 9090 },
                    path: Path {
                        raw_path: "/".to_string(),
                        ..Default::default()
                    },
                },
                headers: HeaderMap {
                    raw: HashMap::from([
                        ("host".to_string(), "localhost:9090".to_string()),
                    ]),
                    size: 18,
                    ..Default::default()
                },
            },
            body: None,
            stream: None,
            hasbody: false,
        }
    )]
    fn test_parse_request(#[case] input: Vec<&str>, #[case] expected: Request<Cursor<String>>) {
        let input = input.join("\n");
        let mut cursor = Cursor::new(input);

        let (req, _) = Request::parse(&mut cursor).unwrap();
        assert_eq!(req, expected);
    }

    #[test]
    fn test_request_call() {
        let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();

        let req: Request<TcpStream> = Request {
            parts: Parts {
                method: Method::GET,
                standard: Standard {
                    name: "HTTP".to_string(),
                    version: Version {
                        major: 1,
                        minor: Some(1),
                        patch: None,
                    },
                },
                url: Url {
                    scheme: "http".to_string(),
                    authority: Authority::Domain {
                        host: "localhost".to_string(),
                        port: 9090,
                    },
                    path: Path {
                        raw_path: "/".to_string(),
                        ..Default::default()
                    },
                },
                headers: HeaderMap {
                    raw: HashMap::from([("host".to_string(), "127.0.0.1".to_string())]),
                    size: 18,
                    ..Default::default()
                },
            },
            body: None,
            stream: None,
            hasbody: false,
        };

        let resp = req.call(&mut stream).unwrap();
        println!("{:?}", resp);
    }
}
