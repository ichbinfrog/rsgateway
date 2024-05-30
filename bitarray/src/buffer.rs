use std::fmt::{Binary, Debug};

use num_traits::{FromBytes, PrimInt, ToBytes, Unsigned};

#[derive(Debug)]
pub enum Error {
    OutOfRange { size: usize, pos: usize },
}

#[derive(Debug)]
pub struct Index {
    pos: usize,
    offset: usize,
    mask: u8,
}

pub struct Buffer {
    data: Vec<u8>,
    bit_size: usize,
    bit_cursor: usize,
}

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
                    self.data[pos] |= *v << offset;
                    self.data[pos + 1] |= *v >> (8 - offset);
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

    pub fn read<T, const N: usize>(&mut self) -> Result<(T, usize), Error>
    where
        T: PrimInt + Unsigned + Binary + FromBytes<Bytes = [u8; N]>,
    {
        let n = std::mem::size_of::<T>();
        if self.bit_cursor + n >= self.bit_size {
            return Err(Error::OutOfRange {
                size: self.bit_size,
                pos: self.bit_cursor + n,
            });
        }

        let mut res = [0u8; N];
        for i in 0..n {
            match self.coord(self.bit_cursor + (i * 8)) {
                Index { pos, offset: 0, .. } => {
                    res[i] = self.data[pos];
                }
                Index { pos, offset, .. } => {
                    let mask: u8 = match offset {
                        1 => 0b11111110,
                        2 => 0b11111100,
                        3 => 0b11111000,
                        4 => 0b11110000,
                        5 => 0b11100000,
                        6 => 0b11000000,
                        7 => 0b10000000,
                        _ => panic!("u8 overflow"),
                    };

                    res[i] = ((self.data[pos] & mask) >> offset)
                        | ((self.data[pos + 1] & !mask) << (8 - offset))
                }
            }
        }

        Ok((T::from_be_bytes(&res), n * 8))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    fn test_simple_push() {
        let mut buf = Buffer::new(32);
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
            vec![0b11111110, 0b11111001, 0b11110111, 0b00001111]
        )
    }

    #[rstest]
    fn test_generic_push() {
        let mut buf = Buffer::new(32);
        assert!(buf.push_bool(false).is_ok());
        assert!(buf.push(u16::MAX).is_ok());

        buf.bit_cursor = 0;
        assert!(buf.skip(1).is_ok());
        assert!(buf
            .read::<u16, 2>()
            .is_ok_and(|(x, n)| x == u16::MAX && n == 16))
    }
}
