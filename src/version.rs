use crate::error::ParseError;
use std::{error::Error, str::FromStr};

#[derive(Debug)]
pub struct Version {
    pub major: usize,
    pub minor: Option<usize>,
    pub patch: Option<usize>,
}

impl FromStr for Version {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_version: Vec<&str> = s.split('.').collect();
        match split_version.len() {
            2 => {
                let (major_raw, minor_raw) = (split_version[0], split_version[1]);

                return Ok(Version {
                    major: usize::from_str_radix(major_raw, 10)?,
                    minor: Some(usize::from_str_radix(minor_raw, 10)?),
                    patch: None,
                });
            }
            3 => {
                let (major_raw, minor_raw, patch_raw) =
                    (split_version[0], split_version[1], split_version[2]);

                return Ok(Version {
                    major: usize::from_str_radix(major_raw, 10)?,
                    minor: Some(usize::from_str_radix(minor_raw, 10)?),
                    patch: Some(usize::from_str_radix(patch_raw, 10)?),
                });
            }
            _ => {}
        }
        Err(ParseError::MalformedStandardVersion.into())
    }
}
