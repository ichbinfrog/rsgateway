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
    traits::TryClone,
};
const MAX_RESPONSE_LINE_SIZE: usize = 8096 * 4;

#[derive(Debug)]
pub struct Response<T> {
    pub standard: Standard,
    pub status: StatusCode,
    pub headers: HeaderMap,

    pub hasbody: bool,
    pub buffer: BufReader<T>,
    pub body: Option<Vec<u8>>,
}

impl<T: TryClone<T> + Read> Response<T> {
    pub fn parse(stream: &mut T) -> Result<Self, Box<dyn Error>> {
        let mut response: Response<T> = Response {
            status: StatusCode::Accepted,
            standard: Standard::default(),
            headers: HeaderMap::default(),
            hasbody: false,
            body: None,
            buffer: BufReader::new(stream.clone()?),
        };

        let mut line = String::with_capacity(MAX_RESPONSE_LINE_SIZE);
        let mut state: u8 = 0;

        loop {
            match response.buffer.read_line(&mut line) {
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

        Ok(response)
    }

    pub fn read_body(&mut self) -> Result<usize, Box<dyn Error>> {
        if self.hasbody {
            match self.headers.get("content-length") {
                Ok(value) => match value {
                    HeaderKind::ContentLength(n) => {
                        let mut body = Vec::new();
                        body.resize(n, 0u8);
                        self.buffer.read_exact(&mut body)?;
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
