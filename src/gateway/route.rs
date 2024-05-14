#[derive(Debug)]
struct Upstream {
    id: String,
    name: String,
    kind: String,
}

#[derive(Debug)]
pub struct Node {
    upstreams: Vec<Upstream>,
    children: Vec<Box<Node>>,
}
