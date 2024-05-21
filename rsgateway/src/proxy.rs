use std::str::FromStr;

use tokio::net::TcpListener;

use dns::resolver::DNS_IP_GOOGLE;
use http::{
    builder::Builder,
    client::Client,
    method::Method,
    request::Request,
    uri::{url::Url},
};

use crate::{
    route::{MatchType, Route},
    trie::Trie,
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
        let mut trie = Trie::new();
        trie.insert(
            "localhost:9090/status",
            Some(Route {
                url: Url::from_str("http://httpbin.org:80/").unwrap(),
            }),
        );

        while let Ok((mut inbound, _)) = self.listener.accept().await {
            let req = Request::parse(&mut inbound).await.unwrap();

            let host = req.parts.url.host().unwrap();
            match trie.get(&host, MatchType::Prefix) {
                None => println!("request did not match any routes {:?}", host),
                Some(upstream) => {
                    tokio::spawn(async move {
                        let proxied_request = Builder::new()
                            .method(Method::POST)
                            .headers(req.parts.headers)
                            .url(upstream.url)
                            .path(req.parts.url.path)
                            .body(req.body)
                            .build();
                        println!("{:?}", proxied_request);

                        let resp = Client::perform(proxied_request, DNS_IP_GOOGLE)
                            .await
                            .unwrap();

                        resp.write(&mut inbound).await.unwrap();
                    });
                }
            }
        }
    }
}
