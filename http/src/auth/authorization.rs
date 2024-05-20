use std::{str::FromStr};

use encoding::base64::{self, STD_ALPHABET};

use crate::error::auth::AuthorizationError;

#[derive(Debug, PartialEq)]
pub enum Authorization {
    Basic { user: String, password: String },
}

impl FromStr for Authorization {
    type Err = AuthorizationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(' ') {
            Some((scheme, rest)) => match scheme.to_lowercase().as_str() {
                "basic" => {
                    let decoded = base64::decode(rest.trim(), STD_ALPHABET)?;
                    match decoded.split_once(':') {
                        Some((user, password)) => {
                            return Ok(Authorization::Basic {
                                user: user.to_string(),
                                password: password.to_string(),
                            })
                        }
                        None => {
                            return Err(AuthorizationError::InvalidFormat {
                                reason: "missing ':' separator between user and password",
                            })
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }

        Err(AuthorizationError::UnknownScheme.into())
    }
}

impl TryFrom<Authorization> for String {
    type Error = AuthorizationError;

    fn try_from(auth: Authorization) -> Result<Self, Self::Error> {
        let mut res = String::new();
        match auth {
            Authorization::Basic { user, password } => {
                res.push_str("Basic ");
                let encoded = base64::encode(&vec![user, password].join(":"), STD_ALPHABET);
                res.push_str(&encoded);
                Ok(res)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==", Authorization::Basic { user: "Aladdin".to_string(), password: "open sesame".to_string() }
    )]
    pub fn test_authorization_from_str(#[case] input: &str, #[case] expected: Authorization) {
        assert_eq!(Authorization::from_str(input).unwrap(), expected);
    }
}
