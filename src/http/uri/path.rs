use crate::encoding::percent;
use std::{collections::HashMap, error::Error, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct Query {
    raw: String,
    lookup: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub raw_path: String,
    pub raw_fragment: Option<String>,
    pub query: Option<Query>,
}

impl FromStr for Query {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut query = Query {
            raw: s.to_string(),
            lookup: HashMap::new(),
        };

        let mut cur = s.to_string().clone();

        while cur.len() > 0 {
            let mut hop: usize = cur.len();
            let mut entry = cur.clone();

            if let Some((e, _)) = cur.split_once('&') {
                hop = e.len() + 1;
                entry = e.to_string();
            }

            if let Some((key, value)) = entry.split_once('=') {
                query
                    .lookup
                    .entry(percent::unescape(key)?)
                    .or_insert(Vec::new())
                    .push(percent::unescape(value)?);
            }

            if hop > cur.len() {
                break;
            }
            cur = cur.split_off(hop);
        }

        Ok(query)
    }
}

impl FromStr for Path {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = Path {
            raw_path: "".to_string(),
            raw_fragment: None,
            query: None,
        };

        let mut raw = s;
        match s.split_once('#') {
            Some((r, f)) => {
                if f.len() > 0 {
                    path.raw_fragment = Some(percent::unescape(f)?);
                } else {
                    path.raw_fragment = None
                }
                raw = r
            }
            None => {}
        }

        match raw.split_once('?') {
            None => {
                path.raw_path = percent::unescape(raw)?;
            }
            Some((raw_path, raw_query)) => {
                path.query = Some(Query::from_str(raw_query)?);
                path.raw_path = percent::unescape(raw_path)?
            }
        }
        Ok(path)
    }
}

impl Default for Path {
    fn default() -> Self {
        Self {
            raw_path: "".to_string(),
            raw_fragment: None,
            query: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("", 
        Query { 
            raw: "".to_string(), 
            lookup: HashMap::new(),
        }
    )]
    #[case("a=a1&a=a2", 
        Query { 
            raw: "a=a1&a=a2".to_string(), 
            lookup: HashMap::from([
                ("a".to_string(), Vec::from(["a1".to_string(), "a2".to_string()])),
            ]) 
        }
    )]
    #[case("a=a1&a=a2&b=b1&c=c1&b=b2&c=c2", 
        Query { 
            raw: "a=a1&a=a2&b=b1&c=c1&b=b2&c=c2".to_string(), 
            lookup: HashMap::from([
                ("a".to_string(), Vec::from(["a1".to_string(), "a2".to_string()])),
                ("b".to_string(), Vec::from(["b1".to_string(), "b2".to_string()])),
                ("c".to_string(), Vec::from(["c1".to_string(), "c2".to_string()])),
            ]) 
        }
    )]
    #[case("a=%3A&b=%26%26", 
        Query { 
            raw: "a=%3A&b=%26%26".to_string(), 
            lookup: HashMap::from([
                ("a".to_string(), Vec::from([":".to_string()])),
                ("b".to_string(), Vec::from(["&&".to_string()])),
            ])
        }
    )]
    fn test_query_parsing(#[case] input: &str, #[case] expected: Query) {
        assert_eq!(Query::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case(
        "/",
        Path { 
            raw_path: "/".to_string(), 
            raw_fragment: None, 
            query: None,
        }
    )]
    #[case(
        "/foo",
        Path { 
            raw_path: "/foo".to_string(), 
            raw_fragment: None, 
            query: None,
        }
    )]
    #[case(
        "/foo#bar",
        Path { 
            raw_path: "/foo".to_string(), 
            raw_fragment: Some("bar".to_string()), 
            query: None,
        }
    )]
    #[case(
        "/foo?a=b#bar",
        Path { 
            raw_path: "/foo".to_string(), 
            raw_fragment: Some("bar".to_string()), 
            query: Some(Query{
                raw: "a=b".to_string(),
                lookup: HashMap::from([
                    ("a".to_string(), Vec::from(["b".to_string()])),
                ]),
            }),
        }
    )]
    fn test_path_parsing(#[case] input: &str, #[case] expected: Path) {
        assert_eq!(Path::from_str(input).unwrap(), expected);
    }
}
