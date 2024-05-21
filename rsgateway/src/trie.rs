use std::collections::HashMap;

use crate::route::{MatchType, Route};

#[derive(Debug)]
pub struct Node {
    upstream: Option<Route>,
    children: HashMap<String, NodeRef>,
}

type NodeRef = Box<Node>;

impl Node {
    pub fn insert(&mut self, prefix: &str, upstream: Option<Route>) {
        match prefix.split_once('/') {
            Some((left, right)) => {
                let mut child = Node {
                    upstream: None,
                    children: HashMap::new(),
                };
                child.insert(right, upstream);
                self.children
                    .entry(left.to_string())
                    .or_insert(child.into());
            }
            _ => {
                let child = Node {
                    upstream,
                    children: HashMap::new(),
                };
                self.children
                    .entry(prefix.to_string())
                    .or_insert(child.into());
            }
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: NodeRef,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: Node {
                upstream: None,
                children: HashMap::new(),
            }
            .into(),
        }
    }

    pub fn insert(&mut self, prefix: &str, upstream: Option<Route>) {
        self.root.insert(prefix, upstream);
    }

    pub fn get(&self, path: &str, kind: MatchType) -> Option<Route> {
        let mut cur = &self.root;
        let mut split = path.split('/');

        for p in split.by_ref() {
            match cur.children.get(p) {
                Some(child) => {
                    cur = child;
                    continue;
                }
                None => {
                    // no longer matches & split.remainder != None
                    match kind {
                        MatchType::Exact => return None,
                        MatchType::Prefix => return cur.upstream.clone(),
                    }
                }
            }
        }

        match &cur.upstream {
            Some(upstream) => {
                // leaf node
                match kind {
                    MatchType::Exact if split.remainder().is_some() => None,
                    _ => Some(upstream.clone()),
                }
            }
            None => {
                // internal node
                None
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use http::uri::{url::Url};

    use super::*;

    #[test]
    fn test_trie() {
        let upstream = Some(Route {
            url: Url::from_str("http://httpbin.org:9090/").unwrap(),
        });
        let mut trie = Trie::new();
        trie.insert("localhost:9090/api/v1", upstream.clone());
        assert_eq!(trie.get("localhost:9090", MatchType::Exact), None);
        assert_eq!(
            trie.get("localhost:9090/api/v1/abc", MatchType::Exact),
            None
        );
        assert_eq!(
            trie.get("localhost:9090/api/v1", MatchType::Exact),
            upstream
        );

        assert_eq!(trie.get("localhost:9090", MatchType::Prefix), None);
        assert_eq!(
            trie.get("localhost:9090/api/v1/", MatchType::Prefix),
            upstream
        );
        assert_eq!(
            trie.get("localhost:9090/api/v1", MatchType::Exact),
            upstream
        );
        assert_eq!(
            trie.get("localhost:9090/api/v1/abc", MatchType::Prefix),
            upstream
        );
    }
}
