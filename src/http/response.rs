use std::{
    error::Error,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use super::{
    error::parse::ParseError,
    header::{HeaderKind, HeaderMap},
    request::Standard,
    statuscode::StatusCode,
};
const MAX_RESPONSE_LINE_SIZE: usize = 8096 * 4;

#[derive(Debug)]
pub struct Response<T> {
    pub standard: Standard,
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
    pub stream: Option<T>,
}

impl<T> Response<T> {
    pub fn parse(stream: &mut T) -> Result<Self, Box<dyn Error>>
    where
        T: Read,
    {
        let mut buf = BufReader::new(stream);
        let mut response: Response<T> = Response {
            status: StatusCode::Accepted,
            standard: Standard::default(),
            headers: HeaderMap::default(),
            body: None,
            stream: None,
        };

        let mut has_body = false;
        let mut line = String::with_capacity(MAX_RESPONSE_LINE_SIZE);
        let mut state: u8 = 0;

        loop {
            match buf.read_line(&mut line) {
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
                    _ => {
                        if line == "\r\n" {
                            has_body = true;
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

        if has_body {
            match response.headers.get("content-length") {
                Ok(value) => match value {
                    HeaderKind::ContentLength(n) => {
                        let mut body = Vec::new();
                        body.resize(n, 0u8);
                        buf.read_exact(&mut body)?;
                        response.body = Some(body);
                    }
                    _ => return Err(ParseError::MissingContentLengthHeader.into()),
                },
                Err(e) => return Err(e.into()),
            }
        }

        Ok((response))
    }
}
