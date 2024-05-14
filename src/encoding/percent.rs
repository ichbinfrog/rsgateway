use crate::http::error::parse::ParseError;
use std::error::Error;
use std::fmt::Write as _;

pub fn escape(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            ':' | '/' | '?' | '#' | '[' | ']' | '@' | '!' | '$' | '&' | '\'' | '(' | ')' | '*'
            | '+' | ',' | ';' | '=' | '%' => {
                res.push('%');
                let _ = write!(res, "{:02X}", ch as u8);
            }
            _ => {
                res.push(ch);
            }
        }
    }

    res
}

pub fn unescape(s: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut res = String::with_capacity(s.len());
    let mut iter = s.chars().enumerate();
    while let Some((i, ch)) = iter.next() {
        match ch {
            '%' => {
                if i + 2 > s.len() - 1 {
                    return Err(ParseError::HexInvalidStringLength { index: i }.into());
                }

                match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    Ok(hex) => {
                        res.push(hex as char);
                    }
                    Err(e) => {
                        return Err(ParseError::HexParseIntError {
                            index: i,
                            kind: e.kind().clone(),
                        }
                        .into())
                    }
                }
                iter.next();
                iter.next();
            }
            '+' => {
                res.push(' ');
            }
            _ => {
                res.push(ch);
            }
        }
    }

    Ok(res)
    // ParseError::MalformedQuery.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case("abc", "abc")]
    #[case("1%41", "1A")]
    #[case("1%41%42%43", "1ABC")]
    #[case("%4a", "J")]
    #[case("%6F", "o")]
    #[case("a+b", "a b")]
    #[case("a%20b", "a b")]
    fn test_unescape(#[case] input: &str, #[case] expected: String) {
        assert_eq!(unescape(input).unwrap(), expected);
    }

    #[rstest]
    #[case("%zzzz")]
    #[case("%")]
    #[case("%a")]
    #[case("%1")]
    #[case("%123%45%6")]
    fn test_unescape_invalid_length(#[case] input: &str) {
        let result = unescape(input);
        assert!(result.is_err());
    }

    #[rstest]
    #[case("", "")]
    #[case("abc%", "abc%25")]
    #[case(":a#bc%", "%3Aa%23bc%25")]
    fn test_escape(#[case] input: &str, #[case] output: &str) {
        assert_eq!(escape(input), output)
    }
}
