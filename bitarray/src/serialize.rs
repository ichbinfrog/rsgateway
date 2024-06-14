use std::net::{Ipv4Addr, Ipv6Addr};

use num_traits::Pow;

use crate::buffer::{Buffer, Error, SizedString};

pub trait Serialize {
    type Err;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err>;
}

pub trait Deserialize {
    type Err;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized;
}

impl Serialize for Ipv4Addr {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        self.to_bits().serialize(buf)
    }
}

impl Deserialize for Ipv4Addr {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        let (bits, n) = buf.read_primitive::<u32, 4>()?;
        Ok((Ipv4Addr::from_bits(bits), n))
    }
}

impl<T: Serialize> Serialize for Vec<T> 
where Error: From<<T as Serialize>::Err> {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        let mut n = 0;
        for v in self.iter() {
            n += v.serialize(buf)?;
        }
        Ok(n)
    }
}

impl<T: Serialize> Serialize for Option<T> 
where Error: From<<T as Serialize>::Err> {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        match self {
            Some(x) => Ok(x.serialize(buf)?),
            None => Ok(0),
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> 
where Error: From<<T as Deserialize>::Err> {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        let (res, n) = T::deserialize(buf)?;
        Ok((Some(res), n))
    }
}

impl Serialize for String {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        let mut i: usize = 0;

        for b in self.bytes() {
            buf.push_primitive(b)?;
            i += 1;
        }

        // TODO: chunk to write as u64
        // for chunk in self.as_bytes().chunks(4) {
        // }

        Ok(i)
    }
}

impl<const N: usize> Deserialize for SizedString<N> {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        let (length, _) = match N {
            1 => {
                let (length, length_size) = buf.read_primitive::<u8, 1>()?;
                (length as usize, length_size)
            }
            2 => {
                let (length, length_size) = buf.read_primitive::<u16, 2>()?;
                (length as usize, length_size)
            }
            3..=4 => {
                let (length, length_size) = buf.read_primitive::<u32, 4>()?;
                (length as usize, length_size)
            }
            5..=8 => {
                let (length, length_size) = buf.read_primitive::<u64, 8>()?;
                (length as usize, length_size)
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        };

        let mut value = Vec::<u8>::with_capacity(length);
        for _ in 0..length {
            let (b, _) = buf.read_primitive::<u8, 1>()?;
            value.push(b);
        }
        let res = SizedString::<N>(String::from_utf8(value)?);
        Ok((res, N + length))
    }
}

impl<const N: usize> Serialize for SizedString<N> {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        let n = self.0.len();
        if n >= 2.pow(4 * N) as usize {
            return Err(Error::Overflow {
                size: n,
                max: 2.pow(N) as usize,
            });
        }

        match N {
            1 => {
                buf.push_primitive(n as u8)?;
            }
            2 => {
                buf.push_primitive(n as u16)?;
            }
            3..=4 => {
                buf.push_primitive(n as u32)?;
            }
            5..=8 => {
                buf.push_primitive(n as u64)?;
            }
            _ => {
                unimplemented!("Serialization for string lengths over u64 has not been implemented")
            }
        }

        self.0.serialize(buf)?;
        Ok(n + N)
    }
}

impl Serialize for u8 {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        buf.push_primitive(*self)
    }
}
impl Deserialize for u8 {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_primitive::<u8, 1>()
    }
}

impl Serialize for u16 {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        buf.push_primitive(*self)
    }
}

impl Deserialize for u16 {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_primitive::<u16, 2>()
    }
}

impl Serialize for u32 {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        buf.push_primitive(*self)
    }
}
impl Deserialize for u32 {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_primitive::<u32, 4>()
    }
}

impl Serialize for u64 {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        buf.push_primitive(*self)
    }
}
impl Deserialize for u64 {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_primitive::<u64, 8>()
    }
}

impl Serialize for bool {
    type Err = Error;
    fn serialize(&self, buf: &mut Buffer) -> Result<usize, Self::Err> {
        buf.push_bool(*self)
    }
}
impl Deserialize for bool {
    type Err = Error;
    fn deserialize(buf: &mut Buffer) -> Result<(Self, usize), Self::Err>
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
        assert!(sized_input.serialize(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<1>::deserialize(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 1 + input.len());

        // Double byte length
        let mut buf: Buffer = Buffer::new(512);
        let sized_input = SizedString::<2>(input.to_string());
        assert!(sized_input.serialize(&mut buf).is_ok());

        buf.bit_cursor = 0;
        let (res, n) = SizedString::<2>::deserialize(&mut buf).unwrap();
        assert_eq!(sized_input, res);
        assert_eq!(n, 2 + input.len());
    }
}
