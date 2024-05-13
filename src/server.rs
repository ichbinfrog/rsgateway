use std::net::TcpListener;

pub const MAX_BUF_LENGTH: usize = 4096;

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(addr: &str) -> Self {
        Self {
            listener: TcpListener::bind(addr).unwrap(),
        }
    }

    pub fn run(&self) {
        loop {
            let (mut stream, _) = self.listener.accept().unwrap();
            // tokio::spawn(async move {
            // let mut buf = String::with_capacity(4096);

            // let r: Request<TcpStream> = Request::from_str().unwrap();
            // match r.parts.method {
            //     Method::POST | Method::PUT => {
            //         if let Ok(body) = r.read_body(&mut reader).await {
            //             println!("{:?} {:?}", r, body);
            //         }
            //     }
            //     _ => {
            //         println!("{:?}", r);
            //     }
            // }

            // let response = "HTTP/1.1 200 OK\r\n\r\n";
            // let response = Response<>
            // stream.write_all(response.as_bytes()).await.unwrap();
            // });
        }
    }
}
