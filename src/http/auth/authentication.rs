use std::{collections::HashMap, str::FromStr};

use crate::http::error::parse::ParseError;

#[derive(Debug)]
pub enum Scheme {
    // https://datatracker.ietf.org/doc/html/rfc7617#section-2
    Basic {
        realm: String,
        charset: Option<String>,
    },
}

impl FromStr for Scheme {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(' ') {
            Some((scheme, rest)) => {
                let mut params: HashMap<&str, String> = HashMap::new();

                for param in rest.split(',') {
                    if let Some((k, v)) = param.split_once('=') {
                        params.insert(k, v.trim_matches('\"').to_string());
                    }
                }

                match scheme.to_lowercase().as_str() {
                    "basic" => match params.get("realm") {
                        Some(v) => {
                            return Ok(Scheme::Basic {
                                realm: v.to_string(),
                                charset: params.get("charset").cloned(),
                            })
                        }
                        _ => {
                            return Err(ParseError::AuthorizationMissingRequiredParam {
                                subject: "realm",
                            })
                        }
                    },
                    _ => {}
                }
            }
            _ => {}
        }

        Ok(Scheme::Basic {
            realm: "".to_string(),
            charset: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(r#"Basic realm="WallyWorld""#)]
    fn test_scheme_parsing(#[case] input: &str) {
        let scheme = Scheme::from_str(input).unwrap();
        println!("{:?}", scheme);
    }
}
