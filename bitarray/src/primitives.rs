use arbitrary_int::{u13, u3, u4};

use crate::{buffer::Error, serialize::{Deserialize, Serialize}};

impl Serialize for u4 {
    type Err = Error;
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Self::Err> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Deserialize for u4 {
    type Err = Error;
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u4>()
    }
}

impl Serialize for u3 {
    type Err = Error;
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Self::Err> {
        buf.push_arbitrary_u8(*self)
    }
}

impl Deserialize for u3 {
    type Err = Error;
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u8::<u3>()
    }
}

impl Serialize for u13 {
    type Err = Error;
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, Self::Err> {
        buf.push_arbitrary_u16(*self)
    }
}

impl Deserialize for u13 {
    type Err = Error;
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), Self::Err>
    where
        Self: Sized,
    {
        buf.read_arbitrary_u16::<u13>()
    }
}
