use crate::error::parse::ParseError;
use std::{error::Error, str::FromStr};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Version {
    pub major: usize,
    pub minor: Option<usize>,
    pub patch: Option<usize>,
}

impl TryFrom<Version> for String {
    type Error = ParseError;

    fn try_from(version: Version) -> Result<Self, Self::Error> {
        let mut res = String::new();
        res.push_str(&version.major.to_string());

        if let Some(minor) = version.minor {
            res.push('.');
            res.push_str(&minor.to_string());
        }

        if let Some(patch) = version.patch {
            res.push('.');
            res.push_str(&patch.to_string());
        }
        Ok(res)
    }
}
impl FromStr for Version {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_version: Vec<&str> = s.split('.').collect();
        match split_version.len() {
            1 => {
                return Ok(Version {
                    major: str::parse(split_version[0])?,
                    minor: None,
                    patch: None,
                })
            }
            2 => {
                let (major_raw, minor_raw) = (split_version[0], split_version[1]);

                return Ok(Version {
                    major: str::parse(major_raw)?,
                    minor: Some(str::parse(minor_raw)?),
                    patch: None,
                });
            }
            3 => {
                let (major_raw, minor_raw, patch_raw) =
                    (split_version[0], split_version[1], split_version[2]);

                return Ok(Version {
                    major: str::parse(major_raw)?,
                    minor: Some(str::parse(minor_raw)?),
                    patch: Some(str::parse(patch_raw)?),
                });
            }
            _ => {}
        }
        Err(ParseError::MalformedStandardVersion.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("1.10.20", Version { major: 1, minor: Some(10), patch: Some(20) })]
    #[case("1.1", Version { major: 1, minor: Some(1), patch: None })]
    #[case("1", Version { major: 1, minor: None, patch: None })]
    fn test_version_parsing(#[case] input: &str, #[case] expected: Version) {
        assert_eq!(Version::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case("")]
    #[case(" ")]
    #[case("1.10.abc")]
    #[case("abc.10.20")]
    #[case("1.abc.20")]
    #[case("1.0.0.0.0.0")]
    fn test_version_parsing_error(#[case] input: &str) {
        assert!(Version::from_str(input).is_err());
    }
}
