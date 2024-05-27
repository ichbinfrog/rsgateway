use std::{fmt::Display, time::Duration};

use crate::frame::{Frame, FrameError};

#[derive(Debug)]
pub enum Command {
    Hello {
        protover: Option<usize>,
    },

    Get {
        key: String,
    },
    Incr {
        key: String,
    },
    Decr {
        key: String,
    },

    Del {
        keys: Vec<String>,
    },
    Set {
        key: String,
        value: String,
        expire_time: Option<Duration>,
        keep_ttl: bool,
    },
}

#[derive(Debug)]
pub struct Builder {
    frames: Vec<Frame>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
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

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hello { .. } => f.write_str("HELLO"),
            Self::Get { .. } => f.write_str("GET"),
            Self::Incr { .. } => f.write_str("INCR"),
            Self::Decr { .. } => f.write_str("DECR"),

            Self::Del { .. } => f.write_str("DEL"),
            Self::Set { .. } => f.write_str("SET"),
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
            Command::Set {
                key,
                value,
                expire_time,
                keep_ttl,
            } => {
                builder.bulk_string(key);
                builder.bulk_string(value);
                if let Some(expire_time) = expire_time {
                    builder.bulk_string("PX");
                    builder.bulk_string(expire_time.as_millis());
                }
                if keep_ttl {
                    builder.bulk_string("KEEPTTL");
                }
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
