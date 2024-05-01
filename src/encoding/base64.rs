const LOOKUP: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[inline]
pub fn encode(input: String) -> String {
    input
        .chars()
        .collect::<Vec<char>>()
        .chunks(3)
        .flat_map(|x| encode_window(x))
        .collect::<String>()
}

#[inline]
fn encode_window(window: &[char]) -> [char; 4] {
    let mut binary: [usize; 24] = [0; 24];
    let mut encoded: [char; 4] = ['=', '=', '=', '='];

    for i in 0..window.len() {
        let res: Vec<usize> = (0..8)
            .rev()
            .map(|n| (window[i] as usize >> n) & 1)
            .collect();
        binary[(i * 8)..((i + 1) * 8)].copy_from_slice(&res);
    }

    for k in 0..4 {
        let mut dec = 0;
        for j in ((k * 6)..((k + 1) * 6)).rev() {
            let exp = 5 - (j - (k * 6));
            dec += &binary[j] * (2 as usize).pow(u32::try_from(exp).unwrap());
        }

        if dec == 0 {
            continue;
        }

        if let Some(ch) = LOOKUP.chars().nth(dec) {
            encoded[k] = ch;
        }
    }

    encoded
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
        assert_eq!(encode(input.to_string()), expected.to_string());
    }
}
