use num_traits::PrimInt;

use crate::serialize::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct u4(pub(crate) u8);

impl Serialize for u4 {
    fn serialize(&self, buf: &mut crate::buffer::Buffer) -> Result<usize, crate::buffer::Error> {
        buf.push_u4(*self)
    }
}

impl Deserialize for u4 {
    fn deserialize(buf: &mut crate::buffer::Buffer) -> Result<(Self, usize), crate::buffer::Error>
    where
        Self: Sized,
    {
        buf.read_u4()
    }
}
