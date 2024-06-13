use std::{
    fmt::{Binary, Debug},
    ops::{Shl, Shr},
    string::FromUtf8Error,
};

use arbitrary_int::{u2, u3, u4, u5, u6, u7, Number};
use num_traits::{AsPrimitive, FromBytes, PrimInt, ToBytes, Unsigned};

#[derive(Debug)]
pub enum Error {
    OutOfRange { size: usize, pos: usize },
    Overflow { size: usize, max: usize },
    Utf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::Utf8Error(value)
    }
}

#[derive(Debug)]
pub struct Index {
    pub(crate) pos: usize,
    pub(crate) offset: usize,
    pub(crate) mask: u8,
}

pub struct Buffer {
    pub(crate) data: Vec<u8>,
    bit_size: usize,
    pub(crate) bit_cursor: usize,
}

#[derive(Debug, PartialEq)]
pub struct SizedString<const N: usize>(pub String);

impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = &self.data;

        for (i, n) in v.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{:08b}", n)?;
        }

        Ok(())
    }
}

fn get_u8_mask(n: usize) -> u8 {
    match n {
        0 => 0,
        1 => 0b1,
        2 => u2::MASK,
        3 => u3::MASK,
        4 => u4::MASK,
        5 => u5::MASK,
        6 => u6::MASK,
        7 => u7::MASK,
        _ => unreachable!("wtf is {}", n),
    }
}

impl Buffer {
    const BYTE: usize = 8;

    pub fn new(cap: usize) -> Self {
        Self {
            bit_size: cap,
            data: vec![0u8; cap / Self::BYTE],
            bit_cursor: 0,
        }
    }

    pub fn from_vec(cap: usize, vec: Vec<u8>) -> Self {
        let n = vec.len();
        Self {
            bit_size: cap,
            data: vec,
            bit_cursor: n,
        }
    }

    pub fn reset(&mut self) {
        self.bit_cursor = 0;
    }

    fn set_bool(&mut self, i: usize, flag: bool) -> Result<Index, Error> {
        if i >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: i,
            });
        }
        let index = self.coord(i);

        if flag {
            self.data[index.pos] |= index.mask;
        } else {
            self.data[index.pos] &= !index.mask;
        }
        Ok(index)
    }

    fn get_bool(&self, i: usize) -> Result<bool, Error> {
        if i >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: i,
            });
        }

        let index = self.coord(i);
        Ok(self.data[index.pos] == (self.data[index.pos] | index.mask))
    }

    fn coord(&self, n: usize) -> Index {
        let offset = n % Self::BYTE;
        let mask = match offset {
            0 => 0b10000000,
            1 => 0b01000000,
            2 => 0b00100000,
            3 => 0b00010000,
            4 => 0b00001000,
            5 => 0b00000100,
            6 => 0b00000010,
            7 => 0b00000001,
            _ => panic!("u8 overflow"),
        };

        Index {
            pos: n / 8,
            offset,
            mask,
        }
    }

    pub fn read_bool(&mut self) -> Result<(bool, usize), Error> {
        let res = self.get_bool(self.bit_cursor)?;
        self.bit_cursor += 1;
        Ok((res, 1))
    }

    pub fn push_bool(&mut self, val: bool) -> Result<usize, Error> {
        if self.bit_cursor + 1 >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + 1,
            });
        }

        if val {
            self.set_bool(self.bit_cursor, val)?;
        }
        self.bit_cursor += 1;
        Ok(1)
    }

    pub fn push_primitive<T>(&mut self, val: T) -> Result<usize, Error>
    where
        T: PrimInt + Unsigned + Binary + ToBytes,
    {
        let n = std::mem::size_of::<T>();
        if self.bit_cursor + n >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }

        for v in val.to_be_bytes().as_ref() {
            match self.coord(self.bit_cursor) {
                Index { pos, offset: 0, .. } => {
                    self.data[pos] = *v;
                }
                Index { pos, offset, .. } => {
                    self.data[pos] |= *v >> offset;
                    self.data[pos + 1] |= *v << (8 - offset);
                }
            }
            self.bit_cursor += 8;
        }
        Ok(n * 8)
    }

    pub fn skip(&mut self, n: usize) -> Result<usize, Error> {
        if self.bit_cursor + n >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }
        self.bit_cursor += n;
        Ok(n)
    }

    pub fn push_arbitrary_u8<T>(&mut self, raw: T) -> Result<usize, Error>
    where
        T: Number<UnderlyingType = u8>,
    {
        let n = T::BITS;
        if self.bit_cursor + n >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }

        let split_bit = Self::BYTE - n;
        let index = self.coord(self.bit_cursor);
        let val = raw.value();

        if index.offset <= split_bit {
            self.data[index.pos] |= val << (split_bit - index.offset);
        } else {
            let mask = get_u8_mask(n);
            let step = index.offset - split_bit;
            let left: u8 = val.shr(step) & mask;
            let right: u8 = val.shl(Self::BYTE - step) & (mask << split_bit);

            self.data[index.pos] |= left;
            self.data[index.pos + 1] |= right;
        }

        self.bit_cursor += n;
        Ok(n)
    }

    pub fn read_arbitrary_u8<T>(&mut self) -> Result<(T, usize), Error>
    where
        T: Number<UnderlyingType = u8>,
    {
        let n = T::BITS;
        if self.bit_cursor + n >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }

        let split_bit = Self::BYTE - n;
        let index = self.coord(self.bit_cursor);
        if index.offset <= split_bit {
            self.bit_cursor += n;
            Ok((
                T::new(self.data[index.pos] >> (split_bit - index.offset)),
                n,
            ))
        } else {
            let step = index.offset - split_bit;
            let left = (self.data[index.pos] & get_u8_mask(n - step)) << step;
            let right = (self.data[index.pos + 1] >> (Self::BYTE - step)) & get_u8_mask(step);
            self.bit_cursor += n;
            Ok((T::new(left | right), n))
        }
    }

    pub fn read_primitive<T, const N: usize>(&mut self) -> Result<(T, usize), Error>
    where
        T: 'static + PrimInt + Unsigned + Binary + FromBytes<Bytes = [u8; N]>,
        u8: AsPrimitive<T>,
    {
        let n = std::mem::size_of::<T>();
        let end = self.bit_cursor + (n * 8);
        if end >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }

        let mut res = T::zero();
        loop {
            if self.bit_cursor >= end {
                break;
            }
            let index = self.coord(self.bit_cursor);
            match index {
                Index { pos, offset: 0, .. } => {
                    if self.bit_cursor + Self::BYTE > end {
                        let sh = end - self.bit_cursor;
                        let mask = match sh {
                            1 => 0b10000000,
                            2 => 0b11000000,
                            3 => 0b11100000,
                            4 => 0b11110000,
                            5 => 0b11111000,
                            6 => 0b11111100,
                            7 => 0b11111110,
                            8 => 0b11111111,
                            _ => unimplemented!("{}", sh),
                        };
                        res = res | ((self.data[pos] & mask).as_() >> (Self::BYTE - sh));
                    } else {
                        let sh = end - (self.bit_cursor + Self::BYTE);
                        res = res | (self.data[pos].as_() << sh);
                    }
                }
                Index { pos, offset, .. } => {
                    let mask = get_u8_mask(Self::BYTE - n);
                    res = res
                        | ((self.data[pos] & mask).as_()
                            << (end - (self.bit_cursor + Self::BYTE - offset)));
                }
            }
            let diff = end - self.bit_cursor;
            let step = if diff < Self::BYTE {
                diff
            } else {
                Self::BYTE - index.offset
            };
            self.bit_cursor += step;
        }

        Ok((res, n * 8))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use arbitrary_int::u7;
    use rstest::*;

    #[rstest]
    fn test_simple_push() {
        let mut buf: Buffer = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_primitive(255u8).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_primitive(255u8).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_primitive(255u8).is_ok());
        assert_eq!(buf.bit_cursor, 1 + 8 + 1 + 1 + 8 + 1 + 8);
        assert_eq!(
            buf.data,
            vec![0b01111111, 0b10011111, 0b11101111, 0b11110000]
        )
    }

    #[rstest]
    fn test_generic_push() {
        let mut buf = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_primitive(u16::MAX).is_ok());

        buf.reset();
        assert!(buf.skip(2).is_ok());
        let (v, n) = buf.read_primitive::<u16, 2>().unwrap();
        assert_eq!(n, 16);
        assert_eq!(v, u16::MAX);
    }

    #[rstest]
    fn test_u4() {
        let mut buf: Buffer = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_bool(false).is_ok());
        let first = u4::new(0b1011u8);
        let second = u4::new(0b1001u8);

        assert!(buf.push_arbitrary_u8(first).is_ok());
        assert!(buf.push_arbitrary_u8(second).is_ok());
        buf.reset();
        assert_eq!((false, 1), buf.read_bool().unwrap());
        assert_eq!((false, 1), buf.read_bool().unwrap());
        assert_eq!((first, 4), buf.read_arbitrary_u8::<u4>().unwrap());
        assert_eq!((second, 4), buf.read_arbitrary_u8::<u4>().unwrap());
    }

    #[rstest]
    #[case(0, u7::new(0b1111001), vec![
        0b011110010, 0b00000000,
    ])]
    #[case(1, u7::new(0b1111001), vec![
        0b01111001, 0b00000000,
    ])]
    #[case(2, u7::new(0b1111001), vec![
        0b00111100, 0b10000000,
    ])]
    #[case(3, u7::new(0b1111001), vec![
        0b00011110, 0b01000000,
    ])]
    #[case(4, u7::new(0b1111001), vec![
        0b00001111, 0b00100000,
    ])]
    #[case(5, u7::new(0b1111001), vec![
        0b00000111, 0b10010000,
    ])]
    #[case(6, u7::new(0b1111001), vec![
        0b00000011, 0b11001000,
    ])]
    #[case(7, u7::new(0b1111001), vec![
        0b00000001, 0b11100100,
    ])]
    #[case(8, u7::new(0b1111001), vec![
        0b00000000, 0b11110010,
    ])]
    fn test_arbitrary_u7(#[case] offset: usize, #[case] input: u7, #[case] layout: Vec<u8>) {
        let mut buf: Buffer = Buffer::new(16);
        for _ in 0..offset {
            assert!(buf.push_bool(false).is_ok());
        }
        assert_eq!(buf.push_arbitrary_u8(input).unwrap(), 7);
        assert_eq!(buf.data, layout);

        buf.reset();
        assert!(buf.skip(offset).is_ok());
        let (res, read) = buf.read_arbitrary_u8::<u7>().unwrap();
        assert_eq!(read, 7);
        assert_eq!(res, input);
    }

    #[rstest]
    #[case(0, u3::new(0b101), vec![
        0b10100000, 0b00000000,
    ])]
    #[case(1, u3::new(0b101), vec![
        0b01010000, 0b00000000,
    ])]
    #[case(2, u3::new(0b101), vec![
        0b00101000, 0b00000000,
    ])]
    #[case(3, u3::new(0b101), vec![
        0b00010100, 0b00000000,
    ])]
    #[case(4, u3::new(0b101), vec![
        0b00001010, 0b00000000,
    ])]
    #[case(5, u3::new(0b101), vec![
        0b00000101, 0b00000000,
    ])]
    #[case(6, u3::new(0b101), vec![
        0b00000010, 0b10000000,
    ])]
    fn test_arbitrary_u3(#[case] offset: usize, #[case] input: u3, #[case] layout: Vec<u8>) {
        let mut buf: Buffer = Buffer::new(16);
        for _ in 0..offset {
            assert!(buf.push_bool(false).is_ok());
        }
        assert_eq!(buf.push_arbitrary_u8(input).unwrap(), 3);
        assert_eq!(buf.data, layout);

        buf.reset();
        assert!(buf.skip(offset).is_ok());
        let (res, read) = buf.read_arbitrary_u8::<u3>().unwrap();
        assert_eq!(read, 3);
        assert_eq!(res, input);
    }
}
