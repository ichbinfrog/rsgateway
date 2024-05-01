use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
struct Upstream {
    id: String,
    name: String,
    kind: String,
}

#[derive(Debug)]
pub struct Node {
    children: HashMap<String, Vec<Rc<RefCell<Node>>>>,
}
