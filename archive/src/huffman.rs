use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap},
    fmt::Debug,
    hash::Hash,
};

use itertools::Itertools;

pub struct Node<T> {
    count: usize,
    value: Option<T>,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("count", &self.count)
            .field("value", &self.value)
            .finish()
    }
}

impl<T: Eq + Hash + Copy> Node<T> {
    fn encode(&self, res: &mut HashMap<T, String>, s: String) {
        if let Some(value) = &self.value {
            res.insert(*value, s);
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

impl<T: PartialEq> Ord for Node<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl<T: PartialEq> Eq for Node<T> {}

impl<T: PartialEq> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.count == other.count
    }
}

#[derive(Debug)]
pub struct Tree<T> {
    root: Node<T>,
    lookup: HashMap<T, String>,
}

impl<T: Eq + Hash + Copy> Tree<T> {
    fn new(input: &[T]) -> Tree<T> {
        let characters: HashMap<&T, usize> = input.iter().counts();
        let n = characters.len();

        let mut nodes: BinaryHeap<Reverse<Node<T>>> = BinaryHeap::with_capacity(n);
        for (k, v) in characters {
            nodes.push(Reverse(Node {
                count: v,
                value: Some(*k),
                left: None,
                right: None,
            }))
        }

        while nodes.len() > 1 {
            let left = nodes.pop().unwrap();
            let right = nodes.pop().unwrap();

            let inter = Node {
                value: None,
                count: left.0.count + right.0.count,
                left: Some(Box::new(left.0)),
                right: Some(Box::new(right.0)),
            };
            nodes.push(Reverse(inter));
        }

        let root = nodes.pop().unwrap().0;
        let mut lookup: HashMap<T, String> = HashMap::with_capacity(n);
        root.encode(&mut lookup, "".to_string());

        Tree { root, lookup }
    }

    fn encode(&self, input: &[T]) -> String {
        let mut res = String::new();

        for ch in input.iter() {
            res.push_str(self.lookup.get(ch).unwrap());
        }

        res
    }

    fn decode(&self, input: &str) -> Vec<T> {
        let mut res = Vec::<T>::new();
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
            if let Some(value) = cur.value {
                res.push(value);
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
        let tree = Tree::<u8>::new(input.as_bytes());

        let encoded = tree.encode(input.as_bytes());
        assert_eq!(input, String::from_utf8(tree.decode(&encoded)).unwrap());
    }
}
