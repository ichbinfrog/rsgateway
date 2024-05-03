use crate::http::method::Method;
use crate::http::request::Request;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::{io::BufReader, net::TcpListener};

pub const MAX_BUF_LENGTH: usize = 4096;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn new(addr: &str) -> Self {
        Self {
            listener: TcpListener::bind(addr).await.unwrap(),
        }
    }

    pub async fn run(&self) {
        loop {
            let (mut stream, _) = self.listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut has_body = false;
                let mut buf = String::with_capacity(4096);
                let mut reader = BufReader::new(&mut stream);

                loop {
                    match reader.read_line(&mut buf).await {
                        Ok(0) => {}
                        Ok(n) => {
                            if n == 2 && buf.ends_with("\r\n") {
                                has_body = true;
                                break;
                            }
                        }
                        Err(_) => {}
                    }
                }

                let r: Request<BufReader<&mut TcpStream>> =
                    Request::from_str(buf.as_str()).unwrap();
                match r.parts.method {
                    Method::POST | Method::PUT => {
                        if let Ok(body) = r.read_body(&mut reader).await {
                            println!("{:?} {:?}", r, body);
                        }
                    }
                    _ => {
                        println!("{:?}", r);
                    }
                }

                let response = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write_all(response.as_bytes()).await.unwrap();
            });
        }
    }
}
