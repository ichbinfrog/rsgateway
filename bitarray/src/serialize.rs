use num_traits::Pow;

use crate::buffer::{Buffer, Error, SizedString};
use std::slice::Chunks;

pub trait Serialize {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error>;
}

pub trait Deserialize {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized;
}

impl Serialize for String {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let mut i: usize = 0;

        for b in self.bytes() {
            buf.push(b)?;
            i += 1;
        }

        // TODO: chunk to write as u64
        // for chunk in self.as_bytes().chunks(4) {
        // }

        Ok(i)
    }
}

impl<const N: usize> Deserialize for SizedString<N> {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        let (length, length_size) = match N {
            1 => {
                let (length, length_size) = buf.read::<u8, 1>()?;
                (length as usize, length_size)
            }
            2 => {
                let (length, length_size) = buf.read::<u16, 2>()?;
                (length as usize, length_size)
            }
            3..=4 => {
                let (length, length_size) = buf.read::<u32, 4>()?;
                (length as usize, length_size)
            }
            5..=8 => {
                let (length, length_size) = buf.read::<u64, 8>()?;
                (length as usize, length_size)
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        };

        let mut value = Vec::<u8>::with_capacity(length as usize);
        for _ in 0..length {
            let (b, _) = buf.read::<u8, 1>()?;
            value.push(b);
        }
        let res = SizedString::<N>(String::from_utf8(value)?);
        return Ok((res, N + length as usize));
    }
}

impl<const N: usize> Serialize for SizedString<N> {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        let n = self.0.len();
        if n >= 2.pow(4 * N) as usize {
            return Err(Error::Overflow {
                size: n,
                max: 2.pow(N) as usize,
            });
        }

        match N {
            1 => {
                buf.push(n as u8)?;
            }
            2 => {
                buf.push(n as u16)?;
            }
            3..=4 => {
                buf.push(n as u32)?;
            }
            5..=8 => {
                buf.push(n as u64)?;
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        }

        self.0.write(buf)?;
        Ok(n + N)
    }
}

impl Serialize for u8 {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push(*self)
    }
}
impl Deserialize for u8 {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read::<u8, 1>()
    }
}

impl Serialize for u16 {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push(*self)
    }
}
impl Deserialize for u16 {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read::<u16, 2>()
    }
}

impl Serialize for u32 {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push(*self)
    }
}
impl Deserialize for u32 {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read::<u32, 4>()
    }
}

impl Serialize for u64 {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push(*self)
    }
}
impl Deserialize for u64 {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read::<u64, 8>()
    }
}

impl Serialize for bool {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push_bool(*self)
    }
}
impl Deserialize for bool {
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_bool()
    }
}

#[cfg(test)]
pub mod tests {
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
        assert!(sized_input.write(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<1>::read(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 1 + input.len());

        // Double byte length
        let mut buf: Buffer = Buffer::new(512);
        let sized_input = SizedString::<2>(input.to_string());
        assert!(sized_input.write(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<2>::read(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 2 + input.len());
    }
}
