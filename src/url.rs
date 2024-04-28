use crate::error::ParseError;
use std::{collections::HashMap, error::Error, str::FromStr, sync::atomic};

#[derive(Debug)]
pub struct Query {
    raw: String,
    lookup: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct Path {
    raw_path: String,
    raw_fragment: String,
    query: Option<Query>,
}

impl FromStr for Query {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let query = Query {
            raw: s.to_string(),
            lookup: HashMap::new(),
        };

        // let mut cur = s.to_string().clone();
        // while cur.len() > 0 {
        //     match cur.split_once('&') {
        //         Some((entry, rest)) => {
        //             cur = rest.to_string();
        //             match entry.split_once('=') {
        //                 Some((key, value)) => {
        //                 }
        //                 None => {}
        //             }
        //         }
        //         None => {}
        //     }
        // }

        Ok((query))
    }
}

impl FromStr for Path {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut path = Path {
            raw_path: "".to_string(),
            raw_fragment: "".to_string(),
            query: None,
        };

        match s.split_once('#') {
            Some((raw, frag)) => {
                path.raw_fragment = frag.to_string();

                match raw.split_once('?') {
                    None => {
                        path.raw_path = raw.to_string();
                    }
                    Some((raw_path, raw_query)) => {
                        path.query = Some(Query {
                            raw: raw_query.to_string(),
                            lookup: HashMap::new(),
                        });
                        path.raw_path = raw_path.to_string();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        println!("{:?}", Path::from_str("/api?token=abc#test"));
    }
}
