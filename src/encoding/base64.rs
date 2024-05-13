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
fn inverse(ch: char) -> u8 {
    if 'A' <= ch && ch <= 'Z' {
        return ch as u8 - 'A' as u8;
    } else if 'a' <= ch && ch <= 'z' {
        return ch as u8 - 'a' as u8 + 26;
    } else if '0' <= ch && ch <= '9' {
        return 52 + ch as u8 - '0' as u8;
    } else if ch == '+' {
        return 62;
    } else if ch == '/' {
        return 63;
    }
    0
}

#[inline]
pub fn decode(input: &str) -> String {
    let mut res = String::with_capacity((input.len() / 4) * 3);

    for window in input.trim_end_matches('=').as_bytes().chunks(4) {
        let n = window.len();
        let mut decoded = [0u8; 3];

        let indexes: Vec<u8> = window.iter().map(|x| inverse(*x as char)).collect();
        match n {
            1 => {
                decoded[0] = (indexes[0] & 0x7C) << 2;
            }
            2 => {
                decoded[0] = ((indexes[0] & 0x7C) << 2) | ((indexes[1] & 0x30) >> 4);
                decoded[1] = (indexes[1] & 0xF) << 4;
            }
            3 => {
                decoded[0] = ((indexes[0] & 0x7C) << 2) | ((indexes[1] & 0x30) >> 4);
                decoded[1] = ((indexes[1] & 0xF) << 4) | ((indexes[2] & 0x3C) >> 2);
            }
            4 => {
                decoded[0] = ((indexes[0] & 0x7C) << 2) | ((indexes[1] & 0x30) >> 4);
                decoded[1] = ((indexes[1] & 0xF) << 4) | ((indexes[2] & 0x3C) >> 2);
                decoded[2] = ((indexes[2]) << 6) | (indexes[3] & 0x3F);
            }
            _ => {}
        }

        res.push(decoded[0] as char);
        res.push(decoded[1] as char);
        res.push(decoded[2] as char);
    }
    res
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
                encoded[1] = lookup((window[0] & 0xC0) << 4);
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
    #[case("Sund", "U3VuZA==")]
    fn test_encoding(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(encode(input), expected.to_string());
    }

    #[rstest]
    #[case("YQ==")]
    #[case("U3U=")]
    #[case("U3Vu")]
    #[case("U3VuZA==")]
    fn test_decoding(#[case] input: &str) {
        println!("{}", decode(input));
    }
}
