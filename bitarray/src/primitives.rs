use arbitrary_int::{u13, u3, u4};

use crate::{buffer::Error, decode::Decoder, encode::Encoder};

impl Encoder for u4 {
    fn encode(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Error> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Decoder for u4 {
    fn decode(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u4>()
    }
}

impl Encoder for u3 {
    fn encode(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Error> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Decoder for u3 {
    fn decode(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u3>()
    }
}

impl Encoder for u13 {
    fn encode(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Error> {
        buf.push_arbitrary_u16(*self)
    }
}

impl Decoder for u13 {
    fn decode(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u16::<u13>()
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Buffer, SizedString};

    use super::*;
    use rstest::*;

    #[rstest]
    #[case("")]
    #[case("h")]
    #[case("ho")]
    #[case("hol")]
    #[case("holl")]
    #[case("holla")]
    fn test_string_serialization_byte(#[case] input: &str) {
        // Single byte length
        let mut buf: Buffer = Buffer::new(512);
        let sized_input = SizedString::<1>(input.to_string());
        assert!(sized_input.encode(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<1>::decode(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 1 + input.len());

        // Double byte length
        let mut buf: Buffer = Buffer::new(512);
        let sized_input = SizedString::<2>(input.to_string());
        assert!(sized_input.encode(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<2>::decode(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 2 + input.len());
    }
}
