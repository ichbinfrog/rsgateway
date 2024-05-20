use std::str::FromStr;

use crate::{error::frame::FrameError, version::Version};

#[derive(Debug, PartialEq, Clone)]
pub struct Standard {
    pub name: String,
    pub version: Version,
}

impl Default for Standard {
    fn default() -> Self {
        Self {
            name: "HTTP".to_string(),
            version: Version {
                major: 1,
                minor: Some(1),
                patch: None,
            },
        }
    }
}

impl TryFrom<Standard> for String {
    type Error = FrameError;

    fn try_from(standard: Standard) -> Result<Self, Self::Error> {
        let mut res = standard.name;
        res.push('/');
        res.push_str(&String::try_from(standard.version)?);
        Ok(res)
    }
}

impl FromStr for Standard {
    type Err = FrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split('/').collect();
        if split.len() == 2 {
            let (name, version) = (split[0], split[1]);
            return Ok(Standard {
                name: name.to_string(),
                version: Version::from_str(version)?,
            });
        }
        Err(FrameError::Invalid {
            subject: "standard",
            reason: "format should be <name>/<version>",
        })
    }
}
