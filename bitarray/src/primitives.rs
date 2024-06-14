use arbitrary_int::{u13, u3, u4};

use crate::serialize::{Deserialize, Serialize};

impl Serialize for u4 {
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, crate::buffer::Error> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Deserialize for u4 {
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), crate::buffer::Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u4>()
    }
}

impl Serialize for u3 {
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, crate::buffer::Error> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Deserialize for u3 {
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), crate::buffer::Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u3>()
    }
}

impl Serialize for u13 {
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, crate::buffer::Error> {
        buf.push_arbitrary_u16(*self)
    }
}

impl Deserialize for u13 {
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), crate::buffer::Error>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u16::<u13>()
    }
}
