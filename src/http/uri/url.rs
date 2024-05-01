use std::str::FromStr;

use crate::http::{error::parse::ParseError, uri::path::Path};

use super::path::Query;

#[derive(Debug, PartialEq)]
pub struct Url {
    scheme: String,
    path: Option<Path>,
    query: Option<Query>,
    port: Option<usize>,

    authority: Option<String>,
}

impl Default for Url {
    fn default() -> Self {
        Self {
            scheme: "".to_string(),
            path: None,
            query: None,
            port: None,

            authority: None,
        }
    }
}

impl FromStr for Url {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut url = Url::default();

        if let Some((scheme, rest)) = s.split_once(":") {
            url.scheme = scheme.to_string();

            match rest.split_once("//") {
                Some(("", rest)) => match rest.split_once('/') {
                    Some((authority, path)) => {
                        url.authority = Some(authority.to_string());

                        let mut path: String = path.to_owned();
                        path.insert(0, '/');
                        url.path = Some(Path::from_str(path.as_str()).unwrap());
                    }
                    _ => {
                        url.authority = Some(rest.to_string());
                    }
                },
                _ => {
                    // TODO: opaque encoding
                }
            }
        }

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        "https://httpbin.org",
        Url { 
            scheme: "https".to_string(),
            authority: Some("httpbin.org".to_string()),
            ..Default::default()
        }
    )]
    #[case(
        "https://httpbin.org/status",
        Url { 
            scheme: "https".to_string(),
            authority: Some("httpbin.org".to_string()),
            path: Some(Path {
                raw_path: "/status".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    )]
    #[case(
        "https://httpbin.org/",
        Url { 
            scheme: "https".to_string(),
            authority: Some("httpbin.org".to_string()),
            path: Some(Path {
                raw_path: "/".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }
    )]
    #[case(
        "https://httpbin.org/status?a=b",
        Url { 
            scheme: "https".to_string(),
            authority: Some("httpbin.org".to_string()),
            path: Some(Path {
                raw_path: "/status".to_string(),
                query: Some(Query::from_str("a=b").unwrap()),
                ..Default::default()
            }),
            ..Default::default()
        }
    )]
    fn test_url_parsing(#[case] input: &str, #[case] expected: Url) {
        assert_eq!(Url::from_str(input).unwrap(), expected);
    }
}
