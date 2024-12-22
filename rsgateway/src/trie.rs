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
            Some((left, right)) => match self.children.get_mut(left) {
                Some(entry) => {
                    entry.insert(right, upstream);
                }
                None => {
                    let mut child = Node {
                        upstream: None,
                        children: HashMap::new(),
                    };
                    child.insert(right, upstream);
                    self.children.insert(left.to_string(), child.into());
                }
            },
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

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn get(&self, path: &str) -> Option<Route> {
        let mut cur = &self.root;
        let path = match path.split_once('?') {
            Some((left, _)) => left,
            None => path,
        };
        let mut split = path.split('/');

        for p in split.by_ref() {
            match cur.children.get(p) {
                Some(child) => {
                    cur = child;
                    continue;
                }
                None => {
                    // no longer matches & split.remainder != None
                    if let Some(upstream) = &cur.upstream {
                        match upstream.match_type {
                            MatchType::Exact => return None,
                            MatchType::Prefix => return cur.upstream.clone(),
                        }
                    }
                }
            }
        }

        match &cur.upstream {
            Some(upstream) => {
                // leaf node
                match upstream.match_type {
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
mod tests {
    use std::str::FromStr;

    use http::uri::url::Url;

    use super::*;

    #[test]
    fn test_trie_basic_prefixs() {
        let upstream = Some(Route {
            url: Url::from_str("http://httpbin.org:9090/").unwrap(),
            match_type: MatchType::Prefix,
        });
        let mut trie = Trie::new();
        trie.insert("localhost:9090/api/v1", upstream.clone());

        assert_eq!(trie.get("localhost:9090"), None);
        assert_eq!(trie.get("localhost:9090/api/v1/"), upstream);
        assert_eq!(trie.get("localhost:9090/api/v1/abc"), upstream);
    }

    #[test]
    fn test_trie_multiple_path() {
        let upstream = Some(Route {
            url: Url::from_str("http://httpbin.org:9090/").unwrap(),
            match_type: MatchType::Exact,
        });
        let mut trie = Trie::new();
        trie.insert("localhost:9090/status", upstream.clone());
        trie.insert("localhost:9090/bytes", upstream.clone());
        trie.insert("localhost:9090/status/205", upstream.clone());

        assert_eq!(trie.get("localhost:9090/status/500"), None,);
        assert_eq!(trie.get("localhost:9090/status/205"), upstream);
        assert_eq!(trie.get("localhost:9090/bytes/205"), None);
        assert_eq!(trie.get("localhost:9090/bytes"), upstream);
    }
}
