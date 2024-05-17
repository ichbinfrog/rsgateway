pub mod proxy;
pub mod route;

use proxy::Proxy;

#[tokio::main]
async fn main() {
    let proxy = Proxy::new("localhost:9090").await;
    proxy.run().await;
}
