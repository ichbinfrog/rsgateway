#[inline]
fn lookup(index: u8) -> char {
    if index <= 25 {
        (65 + index) as char
    } else if index <= 51 {
        (97 + index - 26) as char
    } else if index <= 61 {
        (48 + index - 52) as char
    } else if index == 62 {
        '+'
    } else if index == 63 {
        '/'
    } else {
        '='
    }
}

#[inline]
pub fn encode(input: &str) -> String {
    let encoded_length = ((4 * input.len() / 3) + 3) & !3;
    let mut res = String::with_capacity(encoded_length);

    for window in input.as_bytes().chunks(3) {
        let n = window.len();
        let mut encoded: [char; 4] = ['=', '=', '=', '='];

        match n {
            1 => {
                encoded[0] = lookup(window[0] >> 2);
                encoded[1] = lookup((window[0] & 0x3F) << 4);
            }
            2 => {
                encoded[0] = lookup(window[0] >> 2);
                encoded[1] = lookup(((window[0] & 0x3F) << 4) | (window[1] >> 4));
                encoded[2] = lookup((window[1] & 0xF) << 2);
            }
            3 => {
                encoded[0] = lookup(window[0] >> 2);
                encoded[1] = lookup(((window[0] & 0x3F) << 4) | (window[1] >> 4));
                encoded[2] = lookup(((window[1] & 0xF) << 2) | (window[2] >> 6));
                encoded[3] = lookup(window[2] & 0x3F);
            }
            _ => {}
        }

        res.push(encoded[0]);
        res.push(encoded[1]);
        res.push(encoded[2]);
        res.push(encoded[3]);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("a", "YQ==")]
    #[case("Su", "U3U=")]
    #[case("Sun", "U3Vu")]
    #[case("Sund", "U3VuZ===")]
    fn test_encoding(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(encode(input), expected.to_string());
    }
}
