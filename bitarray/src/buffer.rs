use std::{
    cmp::max,
    fmt::{Binary, Debug},
    string::FromUtf8Error,
};

use num_traits::{AsPrimitive, FromBytes, PrimInt, ToBytes, Unsigned};

use crate::primitives::u4;

#[derive(Debug)]
pub enum Error {
    OutOfRange { size: usize, pos: usize },
    Overflow { size: usize, max: usize },
    Utf8Error(FromUtf8Error),
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        return Error::Utf8Error(value);
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

impl Buffer {
    pub fn new(cap: usize) -> Self {
        Self {
            bit_size: cap,
            data: vec![0u8; (cap / 8) as usize],
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
        let offset = n % 8;
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

    pub fn push<T>(&mut self, val: T) -> Result<usize, Error>
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

    pub fn push_u4(&mut self, val: u4) -> Result<usize, Error> {
        if self.bit_cursor + 4 >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + 4,
            });
        }
        let val = val.0 & 0b1111;
        match self.coord(self.bit_cursor) {
            Index { pos, offset: 0, .. } => self.data[pos] |= val << 4,
            Index { pos, offset: 1, .. } => self.data[pos] |= val << 3,
            Index { pos, offset: 2, .. } => self.data[pos] |= val << 2,
            Index { pos, offset: 3, .. } => self.data[pos] |= val << 1,
            Index { pos, offset: 4, .. } => self.data[pos] |= val,
            Index { pos, offset: 5, .. } => {
                self.data[pos] |= (val & 0b1110) >> 1;
                self.data[pos + 1] |= (val & 0b0001) << 7;
            }
            Index { pos, offset: 6, .. } => {
                self.data[pos] |= (val & 0b1100) >> 2;
                self.data[pos + 1] |= (val & 0b0011) << 6;
            }
            Index { pos, offset: 7, .. } => {
                self.data[pos] |= (val & 0b1000) >> 2;
                self.data[pos + 1] |= (val & 0b0111) << 5;
            }
            _ => unimplemented!(),
        };
        self.bit_cursor += 4;
        Ok(4)
    }

    pub fn read_u4(&mut self) -> Result<(u4, usize), Error> {
        if self.bit_cursor + 4 >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + 4,
            });
        }

        let res = match self.coord(self.bit_cursor) {
            Index { pos, offset: 0, .. } => {
                (self.data[pos] & 0b11110000) >> 3
            }
            Index { pos, offset: 1, .. } => (self.data[pos] & 0b01111000) >> 3,
            Index { pos, offset: 2, .. } => (self.data[pos] & 0b00111100) >> 2,
            Index { pos, offset: 3, .. } => (self.data[pos] & 0b00011110) >> 1,
            Index { pos, offset: 4, .. } => self.data[pos] & 0b00001111,
            Index { pos, offset: 5, .. } => {
                ((self.data[pos] & 0b00000111) << 1) | ((self.data[pos + 1] & 0b10000000) >> 7)
            }
            Index { pos, offset: 6, .. } => {
                ((self.data[pos] & 0b00000011) << 2) | ((self.data[pos + 1] & 0b11000000) >> 6)
            }
            Index { pos, offset: 7, .. } => {
                ((self.data[pos] & 0b00000001) << 3) | ((self.data[pos + 1] & 0b11100000) >> 5)
            }
            _ => 0,
        };
        self.bit_cursor += 4;
        Ok((u4(res), 4))
    }

    pub fn read<T, const N: usize>(&mut self) -> Result<(T, usize), Error>
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
                    if self.bit_cursor + 8 > end {
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
                        res = res | ((self.data[pos] & mask).as_() >> (8 - sh));
                    } else {
                        let sh = end - (self.bit_cursor + 8);
                        res = res | (self.data[pos].as_() << sh);
                    }
                }
                Index { pos, offset, .. } => {
                    let mask: u8 = match offset {
                        1 => 0b011111111,
                        2 => 0b001111111,
                        3 => 0b000111111,
                        4 => 0b000011111,
                        5 => 0b000001111,
                        6 => 0b000000111,
                        7 => 0b000000011,
                        8 => 0b000000001,
                        _ => panic!("u8 overflow"),
                    };

                    res = res
                        | ((self.data[pos] & mask).as_() << (end - (self.bit_cursor + 8 - offset)));
                }
            }
            let diff = end - self.bit_cursor;
            let step = if diff < 8 { diff } else { 8 - index.offset };
            self.bit_cursor += step;
        }

        Ok((res, n * 8))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_simple_push() {
        let mut buf: Buffer = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push(255u8).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push(255u8).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push(255u8).is_ok());
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
        assert!(buf.push(u16::MAX).is_ok());

        buf.reset();
        assert!(buf.skip(2).is_ok());
        let (v, n) = buf.read::<u16, 2>().unwrap();
        assert_eq!(n, 16);
        assert_eq!(v, u16::MAX);
    }

    #[rstest]
    fn test_u4() {
        let mut buf: Buffer = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push_u4(u4(0b1011u8)).is_ok());
        assert!(buf.push_u4(u4(0b1001u8)).is_ok());
        buf.reset();

        assert_eq!((false, 1), buf.read_bool().unwrap());
        assert_eq!((false, 1), buf.read_bool().unwrap());
        assert_eq!((u4(0b1011), 4), buf.read_u4().unwrap());
        assert_eq!((u4(0b1001), 4), buf.read_u4().unwrap());
    }
}
