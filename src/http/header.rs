use super::{
    error::parse::ParseError,
    method::Method,
    mimetype::MimeType,
    uri::{authority::Authority, url::Url},
    useragent::UserAgent,
};
use std::{
    collections::HashMap, convert::Infallible, error::Error, ops::FromResidual, str::FromStr,
};

pub const MAX_HEADER_SIZE: usize = 8190;
pub const MAX_HEADER_MAP_SIZE: usize = MAX_HEADER_SIZE * 128 * 2;

#[derive(Debug, PartialEq, Clone)]
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

    pub fn get(&self, k: &str) -> Result<HeaderKind, Box<dyn Error + Send + Sync>> {
        let lk = k.to_lowercase();
        match self.raw.get(&lk) {
            Some(v) => Ok(HeaderKind::try_from((lk.as_str(), v.as_str()))?),
            None => Err(ParseError::HeaderNotFound.into()),
        }
    }

    pub fn put(&mut self, k: &str, v: HeaderKind) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.raw.insert(k.to_string(), String::try_from(v)?);
        Ok(())
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

    Host(Authority),
    Referer(Url),
}

fn try_header_string_from_vec<T>(values: Option<Vec<T>>) -> Result<String, ParseError>
where
    String: TryFrom<T>,
    <String as TryFrom<T>>::Error: Into<ParseError>,
    Result<String, ParseError>: FromResidual<Result<Infallible, <String as TryFrom<T>>::Error>>,
{
    let mut res: String = String::new();
    if let Some(values) = values {
        let n = values.len();
        for (i, value) in values.into_iter().enumerate() {
            res.push_str(&String::try_from(value)?);
            if n != 1 && i != n - 1 {
                res.push(',');
            }
        }
    }
    Ok(res)
}

fn try_header_vec_from_string<T>(v: &str) -> Vec<T>
where
    T: FromStr,
{
    v.split(',')
        .filter_map(|x| T::from_str(x.trim()).ok())
        .collect()
}

impl TryFrom<HeaderKind> for String {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from(header: HeaderKind) -> Result<Self, Self::Error> {
        let mut res: String = String::new();

        match header {
            HeaderKind::Age(n) => res = n.to_string(),
            HeaderKind::Allow(allowed) => {
                res = try_header_string_from_vec(allowed)?;
            }
            HeaderKind::Accept(content_types) | HeaderKind::ContentType(content_types) => {
                res = try_header_string_from_vec(content_types)?;
            }
            HeaderKind::ContentLength(n) => {
                res.push_str(&n.to_string());
            }
            HeaderKind::UserAgent(user) => {
                res.push_str(&String::try_from(user)?);
            }
            HeaderKind::Host(authority) => {
                res.push_str(&String::try_from(authority)?);
            }
            HeaderKind::Referer(url) => {
                res.push_str(&String::try_from(url)?);
            }
        }

        Ok(res)
    }
}

impl TryFrom<(&str, &str)> for HeaderKind {
    type Error = Box<dyn Error + Send + Sync>;

    fn try_from((k, v): (&str, &str)) -> Result<Self, Self::Error> {
        match k {
            "age" => Ok(Self::Age(str::parse(v)?)),
            "allow" => Ok(Self::Allow(Some(try_header_vec_from_string::<Method>(v)))),
            "accept" => Ok(Self::Accept(Some(try_header_vec_from_string::<MimeType>(
                v,
            )))),
            "content-type" => Ok(Self::ContentType(Some(try_header_vec_from_string::<
                MimeType,
            >(v)))),
            "content-length" => Ok(Self::ContentLength(str::parse(v)?)),
            "user-agent" => Ok(Self::UserAgent(UserAgent::from_str(v)?)),
            "authority" => Ok(Self::Host(Authority::from_str(v)?)),
            "referer" => Ok(Self::Referer(Url::from_str(v)?)),
            "host" => Ok(Self::Host(Authority::from_str(v)?)),
            _ => Err(ParseError::HeaderStructuredGetNotImplemented.into()),
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

        let input: Vec<u8> = vec![0; max_header_size + 1];
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
            HeaderKind::Allow(Some(vec![Method::GET, Method::PUT, Method::POST]))
        );
        assert_eq!(
            headers.get("content-length").unwrap(),
            HeaderKind::ContentLength(20)
        );
        assert_eq!(
            headers.get("accept").unwrap(),
            HeaderKind::Accept(Some(vec![
                MimeType::new("application".to_string(), "xhtml+xml".to_string(), None),
                MimeType::new(
                    "*".to_string(),
                    "*".to_string(),
                    Some(("q".to_string(), "0.8".to_string())),
                ),
            ]))
        );

        println!("{:?}", String::try_from(headers).unwrap());
    }
}
