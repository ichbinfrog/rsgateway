use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    io::{BufRead, Cursor},
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum FrameError {
    IncompleteFrame,

    IoError(std::io::Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UTF8ConversionError(FromUtf8Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameError::IncompleteFrame => "stream ended early".fmt(fmt),
            FrameError::IoError(e) => e.fmt(fmt),
            FrameError::ParseIntError(e) => e.fmt(fmt),
            FrameError::ParseFloatError(e) => e.fmt(fmt),
            FrameError::UTF8ConversionError(e) => e.fmt(fmt),
        }
    }
}
impl std::error::Error for FrameError {}

impl From<std::io::Error> for FrameError {
    fn from(_src: std::io::Error) -> Self {
        Self::IoError(_src)
    }
}

impl From<ParseIntError> for FrameError {
    fn from(_src: ParseIntError) -> Self {
        Self::ParseIntError(_src)
    }
}

impl From<ParseFloatError> for FrameError {
    fn from(_src: ParseFloatError) -> Self {
        Self::ParseFloatError(_src)
    }
}

impl From<FromUtf8Error> for FrameError {
    fn from(_src: FromUtf8Error) -> Self {
        Self::UTF8ConversionError(_src)
    }
}

#[derive(Debug, PartialEq)]
pub enum Frame {
    SimpleString(String),
    BulkString(Option<String>),

    SimpleError(String),
    BulkError(Option<String>),

    Null,
    Integer(i64),
    Boolean(bool),
    Double(f64),

    Array(Vec<Frame>),
    Set(HashSet<Frame>),
    Map(HashMap<Frame, Frame>),
}

impl Eq for Frame {}

impl Hash for Frame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Frame::SimpleString(s) => {
                "ss|".hash(state);
                s.hash(state);
            }
            Frame::SimpleError(s) => {
                "se|".hash(state);
                s.hash(state);
            }
            Frame::BulkError(s) => {
                "bse|".hash(state);
                match s {
                    Some(s) => {
                        s.hash(state);
                    }
                    None => {
                        "_".hash(state);
                    }
                }
            }
            Frame::Integer(i) => {
                "i|".hash(state);
                i.hash(state);
            }
            Frame::BulkString(s) => {
                "bss|".hash(state);
                match s {
                    Some(s) => {
                        s.hash(state);
                    }
                    None => {
                        "_".hash(state);
                    }
                }
            }
            Frame::Null => {
                "_".hash(state);
            }
            Frame::Boolean(b) => b.hash(state),
            Frame::Double(f) => f.to_bits().hash(state),
            Frame::Array(v) => {
                "ar|".hash(state);
                v.hash(state);
            }
            Frame::Set(s) => {
                for v in s {
                    v.hash(state);
                }
            }
            Frame::Map(m) => {
                for (k, v) in m {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl Frame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(cursor)? {
            b'+' => {
                let line = get_line(cursor)?;
                let res = String::from_utf8(line.to_vec())?;
                Ok(Frame::SimpleString(res))
            }
            b'-' => {
                let line = get_line(cursor)?;
                let res = String::from_utf8(line.to_vec())?;
                Ok(Frame::SimpleError(res))
            }
            b':' => Ok(Frame::Integer(get_decimal(cursor)?)),
            b'$' => {
                let length = get_decimal(cursor)?;
                if length < 0 {
                    return Ok(Frame::BulkString(None));
                }

                let res = get_exact(cursor, length as usize)?;
                Ok(Frame::BulkString(Some(String::from_utf8(res.to_vec())?)))
            }
            b'#' => Ok(Frame::Boolean(get_bool(cursor)?)),
            b'*' => {
                let length = get_integer(cursor)?;
                let mut res = Vec::<Frame>::with_capacity(length);
                for _ in 0..length {
                    res.push(Frame::parse(cursor)?);
                }
                Ok(Frame::Array(res))
            }
            b'_' => {
                cursor.set_position(cursor.position() + 2);
                Ok(Frame::Null)
            }
            b',' => Ok(Frame::Double(get_double(cursor)?)),
            b'!' => {
                let length = get_decimal(cursor)?;
                if length < 0 {
                    return Ok(Frame::BulkString(None));
                }

                let res = get_exact(cursor, length as usize)?;
                Ok(Frame::BulkError(Some(String::from_utf8(res.to_vec())?)))
            }
            b'~' => {
                let length = get_integer(cursor)?;
                let mut res = HashSet::<Frame>::with_capacity(length);
                for _ in 0..length {
                    res.insert(Frame::parse(cursor)?);
                }
                Ok(Frame::Set(res))
            }
            b'%' => {
                let length = get_integer(cursor)?;
                let mut res = HashMap::<Frame, Frame>::with_capacity(length);
                for _ in 0..length {
                    let key = Frame::parse(cursor)?;
                    let value = Frame::parse(cursor)?;
                    res.insert(key, value);
                }
                Ok(Frame::Map(res))
            }
            x => {
                println!("{:?}", x as char);
                Err(FrameError::IncompleteFrame)
            }
        }
    }
}

fn get_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !cursor.has_data_left()? {
        return Err(FrameError::IncompleteFrame);
    }
    let i = cursor.position() as usize;
    let res = cursor.get_ref()[i];
    cursor.set_position(i as u64 + 1);
    Ok(res)
}

fn get_bool(cursor: &mut Cursor<&[u8]>) -> Result<bool, FrameError> {
    let res: bool;
    match get_u8(cursor)? {
        b't' => res = true,
        b'f' => res = false,
        _ => return Err(FrameError::IncompleteFrame.into()),
    }
    cursor.set_position(cursor.position() + 2);
    Ok(res)
}

fn get_integer(cursor: &mut Cursor<&[u8]>) -> Result<usize, FrameError> {
    let line = get_line(cursor)?;
    let res = usize::from_str(&String::from_utf8(line.to_vec())?)?;
    Ok(res)
}

fn get_decimal(cursor: &mut Cursor<&[u8]>) -> Result<i64, FrameError> {
    let line = get_line(cursor)?;
    let positive: bool;
    let mut start: usize = 0;
    match line[0] {
        b'-' => {
            positive = false;
            start = 1;
        }
        b'+' => {
            positive = true;
            start = 1;
        }
        b'0'..=b'9' => {
            positive = true;
        }
        _ => return Err(FrameError::IncompleteFrame.into()),
    }
    let res = i64::from_str(&String::from_utf8(line[start..].to_vec())?)?;
    if !positive {
        return Ok(-res);
    }
    Ok(res)
}

fn get_line<'a>(cursor: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len() - 1;

    for i in start..end {
        if cursor.get_ref()[i] == b'\r' && cursor.get_ref()[i + 1] == b'\n' {
            cursor.set_position((i + 2) as u64);
            return Ok(&cursor.get_ref()[start..i]);
        }
    }

    Err(FrameError::IncompleteFrame)
}

fn get_exact<'a>(cursor: &mut Cursor<&'a [u8]>, length: usize) -> Result<&'a [u8], FrameError> {
    let start = cursor.position() as usize;
    let res = &cursor.get_ref()[start..start + length];

    if start + length > cursor.get_ref().len() - 1 {
        return Err(FrameError::IncompleteFrame);
    }

    cursor.set_position((start + length) as u64 + 2);
    Ok(res)
}

fn get_double(cursor: &mut Cursor<&[u8]>) -> Result<f64, FrameError> {
    let line = get_line(cursor)?;
    let res = f64::from_str(&String::from_utf8(line.to_vec())?)?;
    Ok(res)
}

#[cfg(test)]
pub mod tests {
    use std::vec;

    use super::*;

    use rstest::*;

    #[rstest]
    #[case(
        "+OK\r\n",
        Frame::SimpleString("OK".to_string())
    )]
    #[case(":0\r\n", Frame::Integer(0))]
    #[case(":1000\r\n", Frame::Integer(1000))]
    #[case(":+0\r\n", Frame::Integer(0))]
    #[case(":+1000\r\n", Frame::Integer(1000))]
    #[case(":-0\r\n", Frame::Integer(0))]
    #[case(
        ":-1000\r\n", Frame::Integer(-1000),
    )]
    #[case("#t\r\n", Frame::Boolean(true))]
    #[case("#f\r\n", Frame::Boolean(false))]
    #[case("$5\r\nhello\r\n", Frame::BulkString(Some("hello".to_string())))]
    #[case("*0\r\n", Frame::Array(vec![]))]
    #[case(
        "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n",
        Frame::Array(vec![
            Frame::BulkString(Some("hello".to_string())),
            Frame::BulkString(Some("world".to_string()))
        ])
    )]
    #[case(
        "*3\r\n:1\r\n:2\r\n:3\r\n",
        Frame::Array(vec![
            Frame::Integer(1),
            Frame::Integer(2),
            Frame::Integer(3)]
        )
    )]
    #[case(
        "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$5\r\nhello\r\n",
        Frame::Array(vec![
            Frame::Integer(1),
            Frame::Integer(2),
            Frame::Integer(3),
            Frame::Integer(4),
            Frame::BulkString(Some("hello".to_string()))
        ]
    ))]
    #[case(
        "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Hello\r\n-World\r\n",
        Frame::Array(vec![
            Frame::Array(vec![Frame::Integer(1), Frame::Integer(2), Frame::Integer(3)]),
            Frame::Array(vec![Frame::SimpleString("Hello".to_string()), Frame::SimpleError("World".to_string())])
        ])
    )]
    #[case(
        "*2\r\n#t\r\n_",
        Frame::Array(vec![
            Frame::Boolean(true), Frame::Null,
        ])
    )]
    #[case(",1.23\r\n", Frame::Double(1.23))]
    #[case(",inf\r\n", Frame::Double(f64::INFINITY))]
    #[case(",-inf\r\n", Frame::Double(-f64::INFINITY))]
    #[case("!21\r\nSYNTAX invalid syntax\r\n", Frame::BulkError(Some("SYNTAX invalid syntax".to_string())))]
    #[case(
        "~5\r\n:1\r\n:1\r\n:2\r\n:1\r\n:5\r\n", Frame::Set(vec![Frame::Integer(5), Frame::Integer(2), Frame::Integer(1)].into_iter().collect()),
    )]
    #[case(
        "%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n", Frame::Map(HashMap::from_iter(vec![
            (Frame::SimpleString("second".to_string()), Frame::Integer(2)),
            (Frame::SimpleString("first".to_string()), Frame::Integer(1))
        ]))
    )]
    fn test_frame_parsing(#[case] input: &str, #[case] expected: Frame) {
        let mut cursor = Cursor::new(input.as_bytes());
        assert_eq!(Frame::parse(&mut cursor).unwrap(), expected);
    }
}
