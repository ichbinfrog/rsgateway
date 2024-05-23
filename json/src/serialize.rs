use std::io::{BufWriter, Write};

use crate::parser::{Node, NumberNode};

pub trait Serialize {
    fn serialize(&self) -> Node;
}

impl Serialize for u32 {
    fn serialize(&self) -> Node {
        Node::Number(NumberNode::I64(i64::from(*self)))
    }
}
