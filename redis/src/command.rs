use crate::frame::{Frame, FrameError};

#[derive(Debug)]
pub enum Command {
    Hello { protover: Option<usize> },

    Get { key: String },
    Incr { key: String },
    Decr { key: String },

    Del { keys: Vec<String> },
    Set { key: String, value: String },
}

#[derive(Debug)]
pub struct Builder {
    frames: Vec<Frame>,
}

impl Builder {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }
    pub fn bulk_string<T>(&mut self, value: T)
    where
        T: ToString,
    {
        self.frames.push(Frame::BulkString(Some(value.to_string())));
    }

    pub fn build(self) -> Frame {
        Frame::Array(self.frames)
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
            Self::Hello { .. } => "HELLO".to_string(),
            Self::Get { .. } => "GET".to_string(),
            Self::Incr { .. } => "INCR".to_string(),
            Self::Decr { .. } => "DECR".to_string(),

            Self::Del { .. } => "DEL".to_string(),
            Self::Set { .. } => "SET".to_string(),
        }
    }
}

impl TryFrom<Command> for Frame {
    type Error = FrameError;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        let mut builder = Builder::new();
        builder.bulk_string(cmd.to_string());

        match cmd {
            Command::Hello { protover } => {
                if let Some(protover) = protover {
                    builder.bulk_string(protover);
                }
            }
            Command::Get { key } | Command::Incr { key } | Command::Decr { key } => {
                builder.bulk_string(key);
            }
            Command::Set { key, value } => {
                builder.bulk_string(key);
                builder.bulk_string(value);
            }
            Command::Del { keys } => {
                for k in keys {
                    builder.bulk_string(k);
                }
            }
        }

        Ok(builder.build())
    }
}

#[derive(Debug)]
pub enum Response {}

#[cfg(test)]
mod tests {

    use rstest::*;

    #[rstest]
    fn test_frame_from_command() {}
}
