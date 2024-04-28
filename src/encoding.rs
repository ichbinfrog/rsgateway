use std::error::Error;
use rstest::rstest;

use crate::error::ParseError;

pub fn unescape(s: &str) -> Result<String, Box<dyn Error>> {
    let mut res = String::with_capacity(s.len());
    let mut iter = s.chars().enumerate();
    while let Some((i, ch)) = iter.next() {
        match ch {
            '%' => {
                if i + 2 > s.len() - 1 {
                    return Err(ParseError::HexInvalidStringLength.into());
                }

                match u8::from_str_radix(&s[i+1..i+3], 16) {
                    Ok(hex) => {
                        res.push(hex as char);
                    }
                    Err(e) => {
                        return Err(ParseError::HexParseIntError(e).into())
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
    use std::num::ParseIntError;

    use super::*;

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
    #[case("%zzzz", ParseError::HexParseIntError.into())]
    fn test_unescape_error(#[case] input: &str, #[case] expected: Box<dyn Error>) {
        let result = unescape("%zzzz");
        assert!(matches!(result, Err(ParseIntError)))
    }
}
