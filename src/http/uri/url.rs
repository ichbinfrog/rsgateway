use std::{error::Error, str::FromStr};

use crate::http::{error::parse::ParseError, uri::path::Path};

use super::authority::Authority;

#[derive(Debug, PartialEq)]
pub struct Url {
    scheme: String,
    authority: Option<Authority>,
    path: Option<Path>,
}

impl Default for Url {
    fn default() -> Self {
        Self {
            scheme: "".to_string(),
            path: None,
            authority: None,
        }
    }
}

impl TryFrom<Url> for String {
    type Error = Box<dyn Error>;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        let mut res = String::new();

        res.push_str(&url.scheme);
        res.push_str("://");
        if let Some(authority) = url.authority {
            res.push_str(&String::try_from(authority)?);
        }

        if let Some(path) = url.path {
            res.push_str(&String::try_from(path)?);
        }
        Ok(res)
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
                        url.authority = Some(Authority::from_str(authority)?);

                        let mut path: String = path.to_owned();
                        path.insert(0, '/');
                        url.path = Some(Path::from_str(path.as_str()).unwrap());
                    }
                    _ => {
                        url.authority = Some(Authority::from_str(rest)?);
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
    use crate::http::uri::path::Query;

    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        "https://httpbin.org",
        Url { 
            scheme: "https".to_string(),
            authority: Some(Authority::Domain("httpbin.org".to_string())),
            ..Default::default()
        }
    )]
    #[case(
        "https://httpbin.org/status",
        Url { 
            scheme: "https".to_string(),
            authority: Some(Authority::Domain("httpbin.org".to_string())),
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
            authority: Some(Authority::Domain("httpbin.org".to_string())),
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
            authority: Some(Authority::Domain("httpbin.org".to_string())),
            path: Some(Path {
                raw_path: "/status".to_string(),
                query: Some(Query::from_str("a=b").unwrap()),
                ..Default::default()
            }),
            ..Default::default()
        }
    )]
    fn test_url_parsing(#[case] input: &str, #[case] expected: Url) {
        let url = Url::from_str(input).unwrap();
        assert_eq!(url, expected);
        assert_eq!(String::try_from(url).unwrap(), input);
    }
}
