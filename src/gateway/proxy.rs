use tokio::net::TcpListener;

use crate::{
    dns::resolver::Google,
    http::{builder::Builder, client::Client, method::Method, request::Request},
};

pub struct Proxy {
    listener: TcpListener,
}

impl Proxy {
    pub async fn new(addr: &str) -> Self {
        Self {
            listener: TcpListener::bind(addr).await.unwrap(),
        }
    }

    pub async fn run(&self) {
        loop {
            let (mut stream, _) = self.listener.accept().await.unwrap();
            tokio::spawn(async move {
                let req = Request::parse(&mut stream).await.unwrap();

                let mut url = "http://httpbin.org".to_string();
                url.push_str(&String::try_from(req.parts.url.path.clone()).unwrap());
                let mut headers = req.parts.headers.clone();
                headers
                    .raw
                    .insert("host".to_string(), "httpbin.org".to_string());

                let proxied_request = Builder::new()
                    .method(Method::POST)
                    .url(&url)
                    .headers(headers)
                    .body(req.body)
                    .build();
                let resp = Client::perform::<Google>(proxied_request).await.unwrap();
                println!("{:?}", resp);
                println!("{:?}", String::from_utf8(resp.body.unwrap()).unwrap());
            });
        }
    }
}
