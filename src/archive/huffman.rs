use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    fmt::Debug,
};

use itertools::Itertools;

pub struct Node {
    count: usize,
    ch: Option<char>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("count", &self.count)
            .field("ch", &self.ch)
            .finish()
    }
}

impl Node {
    fn encode(&self, res: &mut HashMap<char, String>, s: String) {
        if let Some(ch) = self.ch {
            res.insert(ch, s);
        } else {
            if let Some(ref left) = self.left {
                left.encode(res, s.clone() + "0");
            }

            if let Some(ref right) = self.right {
                right.encode(res, s.clone() + "1");
            }
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.ch == other.ch && self.count == other.count
    }
}

#[derive(Debug)]
pub struct Tree {
    root: Node,
    lookup: HashMap<char, String>,
}

impl Tree {
    fn new(input: &str) -> Tree {
        let characters: HashMap<char, usize> = input.chars().counts();
        let n = characters.len();

        let mut nodes: BinaryHeap<Reverse<Node>> = BinaryHeap::with_capacity(n);
        for (k, v) in characters {
            nodes.push(Reverse(Node {
                count: v,
                ch: Some(k),
                left: None,
                right: None,
            }))
        }

        while nodes.len() > 1 {
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();

            let inter = Node {
                ch: None,
                count: left.0.count + right.0.count,
                left: Some(Box::new(left.0)),
                right: Some(Box::new(right.0)),
            };
            nodes.push(Reverse(inter));
        }

        let root = nodes.pop().unwrap().0;
        let mut lookup: HashMap<char, String> = HashMap::with_capacity(n);
        root.encode(&mut lookup, "".to_string());

        Tree { root, lookup }
    }

    fn encode(&self, input: &str) -> String {
        let mut res = String::new();

        for ch in input.chars() {
            res.push_str(self.lookup.get(&ch).unwrap());
        }

        res
    }

    fn decode(&self, input: &str) -> String {
        let mut res = String::new();
        let mut cur = &self.root;

        for ch in input.chars() {
            match ch {
                '0' => {
                    if let Some(ref left) = cur.left {
                        cur = left;
                    }
                }
                '1' => {
                    if let Some(ref right) = cur.right {
                        cur = right;
                    }
                }
                _ => {}
            }
            if let Some(ch) = cur.ch {
                res.push(ch);
                cur = &self.root;
            }
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        let input = "this is an example of a huffman tree";
        let tree = Tree::new(input);

        let encoded = tree.encode(input);
        assert_eq!(input, tree.decode(&encoded));
    }
}
