#![feature(let_chains)]

use rsgateway::gateway::proxy::Proxy;

#[tokio::main]
async fn main() {
    let proxy = Proxy::new("localhost:9090").await;
    proxy.run().await;
}

// ["GET / HTTP/1.1",
// "Host: localhost:9090",
// "User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0",
// "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
// "Accept-Language: en-US,en;q=0.5",
// "Accept-Encoding: gzip, deflate, br",
// "DNT: 1",
// "Connection: keep-alive",
// "Upgrade-Insecure-Requests: 1",
// "Sec-Fetch-Dest: document",
// "Sec-Fetch-Mode: navigate",
// "Sec-Fetch-Site: none",
// "Sec-Fetch-User: ?1"]
