use std::io::Write;

use crate::error::SerializeError;

pub trait Serialize {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), SerializeError>
    where W: Write;
}

impl Serialize for String {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), SerializeError>
        where W: Write {
        writer.write_all(&[b'"'])?;
        writer.write_all(self.as_bytes())?;
        writer.write_all(&[b'"'])?;
        Ok(())
    }
}
