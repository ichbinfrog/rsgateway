use std::{
    io::Write,
    net::{TcpListener, TcpStream}, str::FromStr,
};

use crate::{dns::resolver::Google, http::{builder::{self, Builder}, client::Client, header::HeaderKind, request::Request, uri::url::Url}};

pub struct Proxy {
    listener: TcpListener,
}

impl Proxy {
    pub fn new(addr: &str) -> Self {
        Self {
            listener: TcpListener::bind(addr).unwrap(),
        }
    }

    pub fn run(&self) {
        loop {
            let (mut stream, _) = self.listener.accept().unwrap();
            let mut reader = stream.try_clone().unwrap();

            let (mut req, mut buffer) = Request::parse(&mut stream).unwrap();
            println!("{:?}", req);

            let mut url = "http://httpbin.org".to_string();
            url.push_str(&String::try_from(req.parts.url.path).unwrap());

            let mut headers = req.parts.headers.clone();
            headers.raw.insert("host".to_string(), "httpbin.org".to_string());
            let resp = Client::get::<Google>(url, headers).unwrap();
            println!("{:?}", resp);
            // req.read_body(&mut buffer).unwrap();
            // println!("{:?}", String::from_utf8(req.body.unwrap()).unwrap());
            // let response = "HTTP/1.1 200 OK\r\n\r\n";
            // reader.write_all(response.as_bytes()).unwrap();
        }
    }
}
