
pub mod error;
pub mod header;
pub mod http;
pub mod url;
pub mod version;
pub mod encoding;

use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

fn handle_stream(mut stream: TcpStream) {
    let buf = BufReader::new(&mut stream);

    let request: Vec<_> = buf
        .lines()
        .map(|res| res.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("{:?}", request)
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_stream(stream);
    }
}
