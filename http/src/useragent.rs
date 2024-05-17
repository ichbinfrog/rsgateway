use std::str::FromStr;

use super::{error::parse::ParseError, version::Version};

#[derive(Debug, PartialEq)]
pub struct UserAgent {
    product: String,
    version: Version,
    comment: Option<String>,
}

impl TryFrom<UserAgent> for String {
    type Error = ParseError;

    fn try_from(user: UserAgent) -> Result<Self, Self::Error> {
        let mut res = String::new();
        res.push_str(&user.product);
        res.push('/');

        res.push_str(&String::try_from(user.version)?);
        if let Some(comment) = &user.comment {
            res.push_str(comment);
        }
        Ok(res)
    }
}

impl FromStr for UserAgent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut user = UserAgent {
            product: "".to_string(),
            version: Version {
                major: 0,
                minor: None,
                patch: None,
            },
            comment: None,
        };
        let mut product = s;

        if let Some((first, rest)) = s.trim().split_once(' ') {
            user.comment = Some(rest.to_string());
            product = first;
        }

        if let Some((name, version)) = product.split_once('/') {
            user.product = name.to_string();

            match Version::from_str(version) {
                Ok(v) => user.version = v,
                Err(e) => {
                    return Err(ParseError::InvalidUserAgent {
                        reason: e.to_string(),
                    })
                }
            }
        }

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0",
        UserAgent { 
            product: "Mozilla".to_string(), 
            version: Version { major: 5, minor: Some(0), patch: None }, 
            comment: Some("(X11; Ubuntu; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0".to_string()),
        }
    )]
    fn parse_user_agent(#[case] input: &str, #[case] expected: UserAgent) {
        assert_eq!(UserAgent::from_str(input).unwrap(), expected);
    }
}
