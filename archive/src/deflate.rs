// https://datatracker.ietf.org/doc/html/rfc1951#section-Abstract
#[derive(Debug)]
pub enum Method {
    NoCompression { length: usize },
}

#[derive(Debug)]
pub struct Block {
    last: bool,
    method: Method,
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn generate_fixed_huffman_decoder() {
        let n = 288;
        let mut bits = [0u8; 288];
        for i in 0..n {
            match i {
                0..=143 => bits[i] = 8,
                144..=255 => bits[i] = 9,
                256..=279 => bits[i] = 7,
                280..=287 => bits[i] = 8,
                _ => {}
            }
        }
    }
}
