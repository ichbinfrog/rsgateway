use crate::http::error::parse::ParseError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct MimeType {
    kind: String,
    sub: String,

    param: Option<(String, String)>,
}

impl MimeType {
    pub fn new(kind: String, sub: String, param: Option<(String, String)>) -> Self {
        Self { kind, sub, param }
    }
}

impl FromStr for MimeType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((t, rest)) => {
                let mut mime = MimeType {
                    kind: t.to_string(),
                    sub: rest.to_string(),
                    param: None,
                };

                if let Some((sub, param)) = rest.split_once(';') {
                    mime.sub = sub.to_string();

                    match param.trim().split_once('=') {
                        Some((k, v)) => {
                            mime.param = Some((k.to_string(), v.to_string()));
                        }
                        _ => {
                            return Err(ParseError::InvalidMimeType {
                                reason: "missing parameter definition".to_string(),
                            })
                        }
                    }
                }
                return Ok(mime);
            }

            _ => {
                return Err(ParseError::InvalidMimeType {
                    reason: "invalid mimetype, format should be type/subtype;parameter=value"
                        .to_string(),
                })
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
        "image/webp", 
        MimeType { 
            kind: "image".to_string(), 
            sub: "webp".to_string(), 
            param: None 
        }
    )]
    #[case(
        "text/plain;charset=UTF-8", 
        MimeType { 
            kind: "text".to_string(), 
            sub: "plain".to_string(), 
            param: Some(("charset".to_string(), "UTF-8".to_string())) 
        }
    )]
    #[case(
        "text/plain;charset=UTF-8", 
        MimeType { 
            kind: "text".to_string(), 
            sub: "plain".to_string(), 
            param: Some(("charset".to_string(), "UTF-8".to_string())) 
        }
    )]
    #[case(
        "text/plain;charset=", 
        MimeType { 
            kind: "text".to_string(), 
            sub: "plain".to_string(), 
            param: Some(("charset".to_string(), "".to_string())) 
        }
    )]
    #[case(
        "multipart/form-data; boundary=aBoundaryString", 
        MimeType { 
            kind: "multipart".to_string(), 
            sub: "form-data".to_string(), 
            param: Some(("boundary".to_string(), "aBoundaryString".to_string())) 
        }
    )]
    fn test_mime_type_from_str(#[case] input: &str, #[case] expected: MimeType) {
        assert_eq!(MimeType::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case("")]
    #[case("image/webp;")]
    #[case("image/webp; charset")]
    fn test_mime_type_from_str_error(#[case] input: &str) {
        assert!(MimeType::from_str(input).is_err())
    }
}
