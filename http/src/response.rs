use std::{fmt::Debug, str::FromStr};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::standard::Standard;

use super::{
    error::frame::FrameError,
    header::{HeaderKind, HeaderMap},
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
    pub async fn write(self, stream: &mut TcpStream) -> Result<(), FrameError> {
        let standard = String::try_from(self.standard)?;
        let status = String::try_from(self.status)?;
        let mut res = String::new();
        res.push_str(&standard);
        res.push(' ');
        res.push_str(&status);
        res.push_str("\r\n");
        stream.write_all(res.as_bytes()).await?;

        stream
            .write_all(String::try_from(self.headers)?.as_bytes())
            .await?;
        stream.write_all(b"\r\n").await?;

        if let Some(body) = self.body {
            stream.write_all(&body).await?;
        }
        Ok(())
    }

    pub async fn parse(stream: &mut TcpStream) -> Result<Self, FrameError> {
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
            match buffer.read_line(&mut line).await? {
                0 => {
                    break;
                }
                _n => match state {
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
            }
        }

        if response.hasbody && response.status != StatusCode::NoContent {
            if let HeaderKind::ContentLength(n) = response.headers.get("content-length")? {
                let mut body = vec![0; n];
                buffer.read_exact(&mut body).await?;
                response.body = Some(body);
                return Ok(response);
            }
        }

        Ok(response)
    }
}
