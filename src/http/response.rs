use std::{error::Error, fmt::Debug, str::FromStr};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

use super::{
    error::parse::ParseError,
    header::{HeaderKind, HeaderMap},
    request::Standard,
    statuscode::StatusCode,
};
const MAX_RESPONSE_LINE_SIZE: usize = 8096 * 4;

pub struct Response {
    pub standard: Standard,
    pub status: StatusCode,
    pub headers: HeaderMap,

    pub hasbody: bool,
    pub body: Option<Vec<u8>>,
}

impl Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("response")
            .field("standard", &self.standard)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .finish()
    }
}

impl Response {
    pub async fn parse(stream: &mut TcpStream) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut buffer = BufReader::new(stream);
        let mut response: Response = Response {
            status: StatusCode::Accepted,
            standard: Standard::default(),
            headers: HeaderMap::default(),
            hasbody: false,
            body: None,
        };

        let mut line = String::with_capacity(MAX_RESPONSE_LINE_SIZE);
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
                                    0 => response.standard = Standard::from_str(&acc)?,
                                    1 => response.status = StatusCode::from_str(&acc)?,
                                    _ => {
                                        break;
                                    }
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
                    _ => {
                        if line == "\r\n" {
                            response.hasbody = true;
                            break;
                        }
                        let _ = response.headers.parse(&line);
                        line.clear();
                    }
                },
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if response.hasbody {
            match response.headers.get("content-length") {
                Ok(value) => match value {
                    HeaderKind::ContentLength(n) => {
                        let mut body = Vec::new();
                        body.resize(n, 0u8);
                        buffer.read_exact(&mut body).await?;
                        response.body = Some(body);
                        return Ok(response);
                    }
                    _ => return Err(ParseError::MissingContentLengthHeader.into()),
                },
                Err(e) => return Err(e.into()),
            }
        }

        Ok(response)
    }
}
//     pub async fn read_body(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
//         if self.hasbody {
//             match self.headers.get("content-length") {
//                 Ok(value) => match value {
//                     HeaderKind::ContentLength(n) => {
//                         let mut body = Vec::new();
//                         body.resize(n, 0u8);
//                         self.buffer.read_exact(&mut body).await?;
//                         self.body = Some(body);
//                         return Ok(n);
//                     }
//                     _ => return Err(ParseError::MissingContentLengthHeader.into()),
//                 },
//                 Err(e) => return Err(e.into()),
//             }
//         }
//         Ok(0)
//     }
// }
