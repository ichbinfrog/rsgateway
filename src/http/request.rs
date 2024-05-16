use crate::http::error::parse::ParseError;
use crate::http::header::HeaderKind;
use crate::http::method::Method;
use crate::http::version::Version;

use std::error::Error;
use std::fmt::Debug;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use super::header::HeaderMap;
use super::response::Response;
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
    type Error = Box<dyn Error + Send + Sync>;

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
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split('/').collect();
        if split.len() == 2 {
            let (name, version) = (split[0], split[1]);
            return Ok(Standard {
                name: name.to_string(),
                version: Version::from_str(version)?,
            });
        }
        Err(ParseError::InvalidStandard.into())
    }
}

pub struct Request {
    pub parts: Parts,
    pub hasbody: bool,

    pub body: Option<Vec<u8>>,
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("request")
            .field("parts", &self.parts)
            .finish()
    }
}

impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts && self.body == other.body
    }
}

impl Default for Request {
    fn default() -> Self {
        Self {
            parts: Parts {
                method: Method::UNDEFINED,
                standard: Standard::default(),
                url: Url::default(),
                headers: HeaderMap::default(),
            },
            body: None,
            hasbody: false,
        }
    }
}

impl Request {
    pub async fn write(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
        let req = String::try_from(self.parts)?;
        stream.write_all(req.as_bytes()).await?;

        if let Some(body) = self.body {
            stream.write_all(b"\r\n").await?;
            stream.write_all(&body).await?;
        }

        Ok(())
    }

    pub async fn call(
        self,
        stream: &mut TcpStream,
    ) -> Result<Response, Box<dyn Error + Send + Sync>> {
        self.write(stream).await?;
        Response::parse(stream).await
    }

    pub async fn parse(stream: &mut TcpStream) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut buffer = BufReader::new(stream);
        let mut request = Request::default();
        let mut line = String::with_capacity(MAX_REQUEST_LINE_SIZE);
        let mut state: u8 = 0;

        loop {
            match buffer.read_line(&mut line).await {
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

        if let Ok(HeaderKind::Host(authority)) = request.parts.headers.get("host") {
            request.parts.url.authority = authority;
        }

        if request.hasbody {
            match request.parts.headers.get("content-length") {
                Ok(value) => match value {
                    HeaderKind::ContentLength(n) => {
                        let mut body = vec![0u8; n];
                        buffer.read_exact(&mut body).await?;
                        request.body = Some(body);
                        return Ok(request);
                    }
                    _ => return Err(ParseError::MissingContentLengthHeader.into()),
                },
                Err(e) => return Err(e),
            }
        }

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use crate::http::uri::authority::Authority;
    use crate::http::uri::path::Path;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use tokio::net::{TcpListener, TcpStream};

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
                    scheme: "".to_string(),
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
            hasbody: false,
        }
    )]
    #[tokio::test]
    async fn test_parse_request(#[case] input: Vec<&str>, #[case] expected: Request) {
        let listener = TcpListener::bind(("0.0.0.0", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut stream = TcpStream::connect(addr).await.unwrap();

        let input = input.join("\r\n");
        stream.write_all(input.as_bytes()).await.unwrap();

        let req = Request::parse(&mut stream).await.unwrap();
        assert_eq!(req, expected);
    }

    #[tokio::test]
    async fn test_request_call() {
        let mut stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();

        let req: Request = Request {
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
            hasbody: false,
        };

        let resp = req.call(&mut stream).await.unwrap();
    }
}
