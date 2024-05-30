use crate::buffer::{Buffer, Error};

pub trait Serialize {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error>;
    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized;
}

impl Serialize for u8 {
    fn write(&self, buf: &mut Buffer) -> Result<usize, Error> {
        buf.push(*self)
    }

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

    fn read(buf: &mut Buffer) -> Result<(Self, usize), Error>
    where
        Self: Sized,
    {
        buf.read_bool()
    }
}
