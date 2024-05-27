use async_recursion::async_recursion;
use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::{
    command::Command,
    frame::{Frame, FrameError},
};

pub struct Client {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn do_command(&mut self, command: Command) -> Result<Option<Frame>, FrameError> {
        self.write_frame(&Frame::try_from(command)?).await?;
        self.read_frame().await
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(FrameError::ConnectionError);
                }
            }
        }
    }

    pub fn parse_frame(&mut self) -> Result<Option<Frame>, FrameError> {
        let mut buffer = Cursor::new(&self.buffer[..]);

        match Frame::parse(&mut buffer) {
            Ok(frame) => {
                let len = buffer.position() as usize;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(FrameError::IncompleteFrame) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[async_recursion]
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), FrameError> {
        match frame {
            Frame::Null => {
                self.stream.write_all(b"_\r\n").await?;
            }
            Frame::SimpleString(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::SimpleError(s) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::BulkString(None) => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::BulkString(Some(s)) => {
                let n = s.len();
                self.stream.write_u8(b'$').await?;
                self.stream.write_all(n.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::BulkError(None) => {
                self.stream.write_all(b"!-1\r\n").await?;
            }
            Frame::BulkError(Some(s)) => {
                let n = s.len();
                self.stream.write_u8(b'!').await?;
                self.stream.write_all(n.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(i) => {
                self.stream.write_u8(b':').await?;
                self.stream.write_all(i.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Boolean(b) => {
                self.stream.write_u8(b'#').await?;
                match b {
                    true => self.stream.write_u8(b't').await?,
                    false => self.stream.write_u8(b'f').await?,
                }
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Double(f) => {
                self.stream.write_u8(b',').await?;
                self.stream.write_all(f.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Set(values) => {
                let n = values.len();
                self.stream.write_u8(b'~').await?;
                self.stream.write_all(n.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;

                for v in values.iter() {
                    self.write_frame(v).await?;
                }
            }
            Frame::Array(values) => {
                let n = values.len();
                self.stream.write_u8(b'*').await?;
                self.stream.write_all(n.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                for v in values {
                    self.write_frame(v).await?;
                }
            }
            Frame::Map(dict) => {
                let n = dict.len();
                self.stream.write_u8(b'%').await?;
                self.stream.write_all(n.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
                for (k, v) in dict {
                    self.write_frame(k).await?;
                    self.write_frame(v).await?;
                }
            }
        }
        self.stream.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_redis_client() {
        let stream = TcpStream::connect(env::var("REDIS_HOST").unwrap())
            .await
            .unwrap();
        let mut client = Client::new(stream);

        let key = "test_redis_client_k1".to_string();
        let value = "test_redis_client_v1".to_string();

        let res = client
            .do_command(Command::Hello { protover: Some(3) })
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(res, Frame::Map(_)));

        let res = client
            .do_command(Command::Del {
                keys: vec![key.clone()],
            })
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(res, Frame::Integer(_)));

        let res = client
            .do_command(Command::Get { key: key.clone() })
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(res, Frame::Null));

        let res = client
            .do_command(Command::Set {
                key: key.clone(),
                value: value.clone(),
                expire_time: None,
                keep_ttl: false,
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(res, Frame::SimpleString("OK".to_string()));

        let res = client
            .do_command(Command::Get { key: key.clone() })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(res, Frame::BulkString(Some(value)));

        let res = client
            .do_command(Command::Del {
                keys: vec![key.clone()],
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(res, Frame::Integer(1));
    }
}
