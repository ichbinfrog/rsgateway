use super::{error::parse::ParseError, method::Method, mimetype::MimeType, useragent::UserAgent};
use std::{collections::HashMap, str::FromStr};

pub const MAX_HEADER_SIZE: usize = 8190;
pub const MAX_HEADER_MAP_SIZE: usize = MAX_HEADER_SIZE * 128 * 2;

#[derive(Debug, PartialEq)]
pub struct HeaderMap {
    pub raw: HashMap<String, String>,
    pub size: usize,

    pub max_header_size: usize,
    pub max_total_length: usize,
}

impl Default for HeaderMap {
    fn default() -> Self {
        Self {
            raw: HashMap::new(),
            size: 0,

            max_header_size: MAX_HEADER_SIZE,
            max_total_length: MAX_HEADER_MAP_SIZE,
        }
    }
}

impl TryFrom<HeaderMap> for String {
    type Error = ParseError;

    fn try_from(headers: HeaderMap) -> Result<Self, Self::Error> {
        let mut res = String::new();

        for (k, v) in headers.raw {
            res.push_str(&k);
            res.push(':');
            res.push_str(&v);
            res.push_str("\r\n");
        }

        Ok(res)
    }
}

impl HeaderMap {
    pub fn parse(&mut self, s: &str) -> Result<(), ParseError> {
        if let Some((k, v)) = s.split_once(':') {
            if k.len() > self.max_header_size {
                return Err(ParseError::ContentTooLarge {
                    subject: "header_key".to_string(),
                });
            }

            if v.len() > self.max_header_size {
                return Err(ParseError::ContentTooLarge {
                    subject: "header_value".to_string(),
                });
            }

            if self.size + v.len() + k.len() >= self.max_total_length {
                return Err(ParseError::ContentTooLarge {
                    subject: "header_map".to_string(),
                });
            }

            let lk = k.trim();
            let lv = v.trim();

            self.size += lv.len() + lk.len();
            self.raw.insert(lk.to_lowercase(), lv.to_string());
        }
        Ok(())
    }

    pub fn get(&self, k: &str) -> Result<HeaderKind, ParseError> {
        let lk = k.to_lowercase();
        match self.raw.get(&lk) {
            Some(v) => HeaderKind::try_from((lk.as_str(), v.as_str())),
            None => Err(ParseError::HeaderNotFound),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HeaderKind {
    Age(usize),
    Allow(Option<Vec<Method>>),
    Accept(Option<Vec<MimeType>>),

    ContentLength(usize),
    ContentType(Option<Vec<MimeType>>),
    UserAgent(UserAgent),
}

impl TryFrom<HeaderKind> for String {
    type Error = ParseError;

    fn try_from(header: HeaderKind) -> Result<Self, Self::Error> {
        let mut res = String::new();

        match header {
            HeaderKind::Age(n) => res = n.to_string(),
            HeaderKind::Allow(allowed) => {
                if let Some(methods) = allowed {
                    let n = methods.len();
                    let mut i = 0;

                    for method in methods {
                        res.push_str(&String::try_from(method)?);
                        if n != 1 && i != n - 1 {
                            res.push(',');
                        }
                        i += 1;
                    }
                }
            }
            HeaderKind::Accept(content_types) | HeaderKind::ContentType(content_types) => {
                if let Some(mimetypes) = content_types {
                    let n = mimetypes.len();
                    let mut i = 0;

                    for mimetype in mimetypes {
                        res.push_str(&String::try_from(mimetype)?);
                        if n != 1 && i != n - 1 {
                            res.push(',');
                        }
                        i += 1;
                    }
                }
            }
            HeaderKind::ContentLength(n) => {
                res.push_str(&n.to_string());
            }
            HeaderKind::UserAgent(user) => {
                res.push_str(&String::try_from(user)?);
            }
        }

        Ok(res)
    }
}

impl TryFrom<(&str, &str)> for HeaderKind {
    type Error = ParseError;

    fn try_from((k, v): (&str, &str)) -> Result<Self, Self::Error> {
        match k {
            "age" => match usize::from_str_radix(v, 10) {
                Ok(v) => Ok(Self::Age(v)),
                Err(e) => {
                    return Err(ParseError::InvalidInteger {
                        reason: e.to_string(),
                        subject: "age header",
                    })
                }
            },
            "allow" => Ok(Self::Allow {
                0: Some(
                    v.split(',')
                        .filter_map(|x| Method::from_str(x.trim()).ok())
                        .collect(),
                ),
            }),
            "accept" => Ok(Self::Accept {
                0: Some(
                    v.split(',')
                        .filter_map(|x| MimeType::from_str(x.trim()).ok())
                        .collect(),
                ),
            }),
            "content-type" => Ok(Self::ContentType {
                0: Some(
                    v.split(',')
                        .filter_map(|x| MimeType::from_str(x.trim()).ok())
                        .collect(),
                ),
            }),
            "content-length" => match usize::from_str_radix(v, 10) {
                Ok(v) => Ok(Self::ContentLength(v)),
                Err(e) => {
                    return Err(ParseError::InvalidInteger {
                        reason: e.to_string(),
                        subject: "age header",
                    })
                }
            },
            "user-agent" => Ok(Self::UserAgent(UserAgent::from_str(v)?)),
            _ => Err(ParseError::HeaderStructuredGetNotImplemented),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_parsing_max_size() {
        let max_header_size = 10;
        let max_total_length = 12;

        let mut headers = HeaderMap {
            max_header_size,
            max_total_length,
            ..Default::default()
        };

        let mut input: Vec<u8> = Vec::with_capacity(max_header_size + 1);
        input.resize(max_header_size + 1, 0);
        let long = String::from_utf8(input).unwrap();

        // key > MAX_HEADER_SIZE
        let mut key = long.clone();
        key.push_str(": abc");
        assert!(headers.parse(&key).is_err());

        // value > MAX_HEADER_SIZE
        let mut value = "abc: ".to_string();
        value.push_str(&long);
        assert!(headers.parse(&value).is_err());

        // total > MAX_HEADER_SIZE
        assert!(headers.parse("foo: bar").is_ok());
        assert!(headers.parse("holla: quetal").is_err());
    }

    #[test]
    fn test_headers_parsing() {
        let mut headers = HeaderMap::default();

        assert!(headers.parse("Allow: GET,PUT,POST").is_ok());
        assert!(headers
            .parse("Accept: application/xhtml+xml,*/*;q=0.8")
            .is_ok());
        assert!(headers.parse("Content-Length: 20").is_ok());

        assert_eq!(
            headers.get("allow").unwrap(),
            HeaderKind::Allow {
                0: Some(vec![Method::GET, Method::PUT, Method::POST])
            }
        );
        assert_eq!(
            headers.get("content-length").unwrap(),
            HeaderKind::ContentLength(20)
        );
        assert_eq!(
            headers.get("accept").unwrap(),
            HeaderKind::Accept {
                0: Some(vec![
                    MimeType::new("application".to_string(), "xhtml+xml".to_string(), None),
                    MimeType::new(
                        "*".to_string(),
                        "*".to_string(),
                        Some(("q".to_string(), "0.8".to_string())),
                    ),
                ])
            }
        );

        println!("{:?}", String::try_from(headers).unwrap());
    }
}
