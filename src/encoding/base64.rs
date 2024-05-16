pub const STD_ALPHABET: &[char; 64] = &['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9','+','/'];
pub const URL_ALPHABET: &[char; 64] = &['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9','-','_'];

#[inline]
fn lookup(index: u8, alphabet: &[char; 64]) -> char {
    alphabet[index as usize] as char
}

#[inline]
fn inverse(ch: char, alphabet: &[char; 64]) -> u8 {
    alphabet.iter().position(|&r| r == ch).unwrap() as u8
}

#[inline]
pub fn decode(input: &str, alphabet: &[char; 64]) -> String {
    let mut res = String::with_capacity((input.len() / 4) * 3);

    for window in input.trim_end_matches('=').as_bytes().chunks(4) {
        let n = window.len();
        let indexes: Vec<u8> = window.iter().map(|x| inverse(*x as char, alphabet)).collect();
        match n {
            2 => {
                res.push(((indexes[0] << 2) | ((indexes[1] & 0x30) >> 4)) as char);
            }
            3 => {
                res.push(((indexes[0] << 2) | ((indexes[1] & 0x30) >> 4)) as char);
                res.push((((indexes[1] & 0xF) << 4) | ((indexes[2] & 0x3C) >> 2)) as char);
            }
            4 => {
                res.push(((indexes[0] << 2) | ((indexes[1] & 0x30) >> 4)) as char);
                res.push((((indexes[1] & 0xF) << 4) | ((indexes[2] & 0x3C) >> 2)) as char);
                res.push((((indexes[2]) << 6) | (indexes[3] & 0x3F)) as char);
            }
            _ => {}
        }
    }
    res
}

#[inline]
pub fn encode(input: &str, alphabet: &[char; 64]) -> String {
    let encoded_length = ((4 * input.len() / 3) + 3) & !3;
    let mut res = String::with_capacity(encoded_length);

    for window in input.as_bytes().chunks(3) {
        let n = window.len();
        let mut encoded: [char; 4] = ['=', '=', '=', '='];

        match n {
            1 => {
                encoded[0] = lookup(window[0] >> 2, alphabet);
                encoded[1] = lookup((window[0] & 0x03) << 4, alphabet);
            }
            2 => {
                encoded[0] = lookup(window[0] >> 2, alphabet);
                encoded[1] = lookup(((window[0] & 0x03) << 4) | (window[1] >> 4), alphabet);
                encoded[2] = lookup((window[1] & 0xF) << 2, alphabet);
            }
            3 => {
                encoded[0] = lookup(window[0] >> 2, alphabet);
                encoded[1] = lookup(((window[0] & 0x03) << 4) | (window[1] >> 4), alphabet);
                encoded[2] = lookup(((window[1] & 0xF) << 2) | (window[2] >> 6), alphabet);
                encoded[3] = lookup(window[2] & 0x3F, alphabet);
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
    #[case("f", "Zg==")]
    #[case("fo", "Zm8=")]
    #[case("foo", "Zm9v")]
    #[case("foob", "Zm9vYg==")]
    #[case("fooba", "Zm9vYmE=")]
    #[case("foobar", "Zm9vYmFy")]
    #[case("a", "YQ==")]
    #[case("Su", "U3U=")]
    #[case("Sun", "U3Vu")]
    #[case("Sund", "U3VuZA==")]
    #[case("sure.", "c3VyZS4=")]
	#[case("sure", "c3VyZQ==")]
	#[case("sur", "c3Vy")]
	#[case("su", "c3U=")]
	#[case("leasure.", "bGVhc3VyZS4=")]
	#[case("easure.", "ZWFzdXJlLg==")]
	#[case("asure.", "YXN1cmUu")]
	#[case("sure.", "c3VyZS4=")]
    fn test_std_encoding(#[case] input: &str, #[case] expected: &str) {
        let encoded = encode(input, STD_ALPHABET);
        assert_eq!(encoded, expected.to_string());
        
        assert_eq!(decode(&encoded, STD_ALPHABET), input);
    }
}
