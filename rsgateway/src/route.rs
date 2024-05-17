use http::uri::{authority::Authority, path::Path};

#[derive(Debug)]
pub struct Upstream {
    id: String,
    name: String,
    kind: String,
}

// #[derive(Debug)]
// pub struct Node {
//     upstreams: Vec<Upstream>,
//     children: Vec<Box<Node>>,
// }

#[derive(Debug)]
pub struct Route {
    id: String,
    name: String,
    hosts: Vec<Authority>,
    paths: Vec<Path>,
}
