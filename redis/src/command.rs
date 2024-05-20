use crate::frame::{Frame, FrameError};

#[derive(Debug)]
pub enum Command {
    Hello { protover: Option<usize> },
    Get { key: String },
    Del { keys: Vec<String> },
    Set { key: String, value: String },
}

impl TryFrom<Command> for Frame {
    type Error = FrameError;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Hello { protover } => {
                let mut frames = vec![Frame::BulkString(Some("HELLO".to_string()))];
                if let Some(protover) = protover {
                    frames.push(Frame::BulkString(Some(protover.to_string())));
                }
                Ok(Frame::Array(frames))
            }
            Command::Get { key } => Ok(Frame::Array(vec![
                Frame::BulkString(Some("GET".to_string())),
                Frame::BulkString(Some(key)),
            ])),
            Command::Set { key, value } => Ok(Frame::Array(vec![
                Frame::BulkString(Some("SET".to_string())),
                Frame::BulkString(Some(key)),
                Frame::BulkString(Some(value)),
            ])),
            Command::Del { keys } => {
                let mut frames = Vec::<Frame>::with_capacity(keys.len() + 1);
                frames.push(Frame::BulkString(Some("DEL".to_string())));
                for k in keys {
                    frames.push(Frame::BulkString(Some(k)));
                }
                Ok(Frame::Array(frames))
            }
        }
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
