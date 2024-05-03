use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::http::error::parse::ParseError;
use crate::http::method::Method;
use crate::http::version::Version;

use std::error::Error;
use std::str::FromStr;

use super::header::{HeaderKind, HeaderMap};
use super::uri::path::Path;

#[derive(Debug, PartialEq)]
pub struct Parts {
    pub method: Method,
    pub path: Path,
    pub standard: Standard,

    pub headers: HeaderMap,
}

impl TryFrom<Parts> for String {
    type Error = Box<dyn Error>;

    fn try_from(p: Parts) -> Result<Self, Self::Error> {
        let mut res = String::new();
        res.push_str(&String::try_from(p.method)?);
        res.push(' ');
        res.push_str(&String::try_from(p.path)?);
        res.push(' ');
        res.push_str(&String::try_from(p.standard)?);
        res.push_str("\r\n");
        res.push_str(&String::try_from(p.headers)?);

        Ok(res)
    }
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

impl TryFrom<Standard> for String {
    type Error = ParseError;

    fn try_from(standard: Standard) -> Result<Self, Self::Error> {
        let mut res = standard.name;
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

#[derive(Debug, PartialEq)]
pub struct Request<T> {
    pub parts: Parts,
    pub body: Option<T>,
}

impl<T> Default for Request<T> {
    fn default() -> Self {
        Self {
            parts: Parts {
                method: Method::UNDEFINED,
                standard: Standard::default(),
                path: Path::default(),
                headers: HeaderMap::default(),
            },
            body: None,
        }
    }
}

impl<T> FromStr for Request<T> {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request: Request<T> = Default::default();

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
                                0 => request.parts.method = Method::from_str(&acc)?,
                                1 => request.parts.path = Path::from_str(&acc)?,
                                2 => request.parts.standard = Standard::from_str(&acc)?,
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
                    let _ = request.parts.headers.parse(&line);
                }
            }
        }

        Ok(request)
    }
}

impl<T> Request<T> {
    pub async fn call(self, writer: &mut T) -> Result<(), Box<dyn Error>>
    where
        T: AsyncWrite + Unpin,
    {
        let res: String = String::try_from(self.parts)?;
        writer.write_all(res.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }

    pub async fn read_body(&self, reader: &mut T) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: AsyncBufRead + Unpin,
    {
        let header = self.parts.headers.get("content-length")?;
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
    use std::{collections::HashMap, io::Cursor};

    use super::*;
    use rstest::*;
    use tokio::io::{stdout, Stdout};

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
            },
            body: None,
        }
    )]
    fn test_parse_request(#[case] input: Vec<&str>, #[case] expected: Request<()>) {
        let req: Request<()> = Request::from_str(input.join("\n").as_str()).unwrap();
        assert_eq!(req, expected);
        println!("{:?}", req);
    }

    #[tokio::test]
    async fn test_request_call() {
        let input = vec![
            "GET / HTTP/1.1",
            "Host: localhost:9090",
        ];
        let req: Request<Vec<u8>> = Request::from_str(input.join("\n").as_str()).unwrap();

        let mut out = Vec::<u8>::new();
        println!("{:?}", req.call(&mut out).await.unwrap());
        println!("{:?}", String::from_utf8(out));
    }
}
