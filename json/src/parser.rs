use itertools::{Itertools, PeekingNext};
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, BufWriter, Lines, Write},
    iter::Peekable,
};

use crate::error::ParserError;

#[derive(PartialEq, Clone)]
pub struct Token {
    line: usize,
    start: usize,
    end: usize,
    kind: TokenKind,
}
impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}..{}){:?}",
            self.line, self.start, self.end, self.kind
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Comma,
    Colon,

    BackSlash,
    Quote,

    String(String),
    Number(String),
    Value(String),
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Array(Vec<Node>),
    String(String),
    Number(NumberNode),
    Boolean(bool),
    Null,
    Object(HashMap<String, Node>),
}

#[derive(Debug, PartialEq)]
pub enum NumberNode {
    F64(f64),
    I64(i64),
}

pub fn value_to_node(token: Token) -> Result<Node, ParserError> {
    match token {
        Token {
            kind: TokenKind::Value(ref s),
            ..
        } => match s.as_str() {
            "true" => Ok(Node::Boolean(true)),
            "false" => Ok(Node::Boolean(false)),
            "null" => Ok(Node::Null),
            _ => Err(ParserError {
                token: Some(token.clone()),
                reason: "invalid value node".to_string(),
            }),
        },
        t => Err(ParserError {
            token: Some(t),
            reason: "unknown node type".to_string(),
        }),
    }
}

pub fn number_to_node(input: &str) -> Result<Node, ParserError> {
    #[derive(Clone, Copy)]
    enum State {
        Integer,
        Fraction,
        Exponent,
    }
    let mut input: String = input.to_string();
    input.push(' ');

    let mut state = State::Integer;

    let mut integer: usize = 0;
    let mut fraction: usize = 0;
    let mut exponent: usize = 0;

    for (i, ch) in input.char_indices() {
        match (state, i, ch) {
            (State::Integer, 0, '-') => {
                continue;
            }
            (State::Integer, _, ch) if ch.is_numeric() => {
                integer = i;
            }
            (State::Integer, _, '.') => {
                integer = i;
                state = State::Fraction;
            }
            (State::Fraction, _, ch) if ch.is_numeric() => {
                fraction = i;
            }
            (State::Integer | State::Fraction, _, 'e' | 'E') => {
                fraction = i;
                state = State::Exponent;
            }
            (State::Exponent, _, ch) if ch.is_numeric() => {
                exponent = i;
            }
            (State::Integer, _, ' ') => {
                integer = i;
            }

            (State::Fraction, _, ' ') => {
                fraction = i;
            }
            (State::Exponent, _, ' ') => {
                exponent = i;
            }
            (_, _, _) => {}
        }
    }

    match state {
        State::Integer => Ok(Node::Number(NumberNode::I64(input[..integer].parse()?))),
        State::Fraction => Ok(Node::Number(NumberNode::F64(input[..fraction].parse()?))),
        State::Exponent => Ok(Node::Number(NumberNode::F64(input[..exponent].parse()?))),
    }
}

pub fn take_string<I: Iterator<Item = (usize, char)>>(
    iter: &mut I,
    line: usize,
    index: usize,
) -> Result<Token, ParserError> {
    let mut res = String::new();
    let mut iter: Peekable<&mut I> = iter.peekable();

    while let Some((i, ch)) = iter.next() {
        match ch {
            '\\' => match iter.next() {
                Some((_, ch)) => match ch {
                    'u' => {
                        let mut hex = String::new();
                        while let Some((_, d)) = iter
                            .next_if(|(j, x)| *j <= i + 1 + 6 && x.is_ascii_hexdigit() && *x != '"')
                        {
                            hex.push(d);
                        }

                        let n = hex.len();
                        if !(4..=6).contains(&n) {
                            return Err(ParserError {
                                token: Some(Token {
                                    line,
                                    start: i,
                                    end: i + n,
                                    kind: TokenKind::BackSlash,
                                }),
                                reason: "unicode character should contain 4..6 hex digits"
                                    .to_string(),
                            });
                        }

                        match char::from_u32(u32::from_str_radix(&hex, 16)?) {
                            Some(c) => {
                                res.push(c);
                            }
                            None => {
                                return Err(ParserError {
                                    token: Some(Token {
                                        line,
                                        start: i,
                                        end: i + n,
                                        kind: TokenKind::BackSlash,
                                    }),
                                    reason: "invalid unicode character".to_string(),
                                })
                            }
                        }
                    }
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' => {
                        res.push(ch);
                    }
                    _ => {
                        return Err(ParserError {
                            token: Some(Token {
                                line,
                                start: i,
                                end: i,
                                kind: TokenKind::BackSlash,
                            }),
                            reason: "unclosed escape character \\".to_string(),
                        })
                    }
                },
                _ => {
                    return Err(ParserError {
                        token: Some(Token {
                            line,
                            start: index,
                            end: index + res.len(),
                            kind: TokenKind::BackSlash,
                        }),
                        reason: "unexpected EOF, unclosed escape character \\".to_string(),
                    })
                }
            },
            '"' => {
                return Ok(Token {
                    line,
                    start: index,
                    end: index + res.len(),
                    kind: TokenKind::String(res),
                })
            }
            _ => res.push(ch),
        }
    }
    Err(ParserError {
        token: Some(Token {
            line,
            start: index,
            end: index + res.len(),
            kind: TokenKind::Quote,
        }),
        reason: "unclosed quote \"".to_string(),
    })
}

pub fn take_while<I: Iterator<Item = (usize, char)> + PeekingNext, F: Fn(&char) -> bool>(
    iter: &mut I,
    index: usize,
    f: F,
) -> (usize, usize, String) {
    let (n, res): (Vec<usize>, String) = iter.peeking_take_while(|(_, x)| f(x)).collect();
    let first = match n.first() {
        Some(v) => v,
        None => &index,
    };
    (*first, first + res.len(), res)
}

pub fn tokenize<T: BufRead>(lines: Lines<T>) -> Result<Vec<Token>, ParserError> {
    let mut res = Vec::<Token>::new();

    for (i, line) in lines.enumerate() {
        if let Ok(line) = line {
            let mut iter = line.char_indices().peekable();

            while let Some((j, ch)) = iter.next() {
                match ch {
                    '"' => {
                        res.push(take_string(&mut iter, i, j)?);
                    }
                    '{' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::BraceOpen,
                    }),
                    '}' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::BraceClose,
                    }),
                    '[' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::BracketOpen,
                    }),
                    ']' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::BracketClose,
                    }),
                    ':' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::Colon,
                    }),
                    ',' => res.push(Token {
                        line: i,
                        start: j,
                        end: j + 1,
                        kind: TokenKind::Comma,
                    }),
                    ch if ch.is_numeric() || ch == '-' => {
                        let (start, end, mut s) = take_while(&mut iter, j, |x| {
                            x.is_numeric()
                                || *x == '.'
                                || *x == 'e'
                                || *x == 'E'
                                || *x == '-'
                                || *x == '+'
                        });
                        s.insert(0, ch);
                        res.push(Token {
                            line: i,
                            start,
                            end,
                            kind: TokenKind::Number(s),
                        });
                    }
                    ch if ch.is_alphabetic() => {
                        let (start, end, mut s) = take_while(&mut iter, j, |x| x.is_alphabetic());
                        s.insert(0, ch);
                        res.push(Token {
                            line: i,
                            start,
                            end,
                            kind: TokenKind::Value(s),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(res)
}

pub fn parse<'a, I: Iterator<Item = &'a Token>>(
    iter: &mut Peekable<I>,
) -> Result<Option<Node>, ParserError> {
    match iter.next() {
        // Some(token @ Token{
        //     kind: TokenKind::BracketClose, ..
        // }) => {
        //     Err(ParserError {
        //         token: Some(token.clone()),
        //         reason: "unopened ]".to_string(),
        //     })
        // }
        // Some(token @ Token{
        //     kind: TokenKind::BraceClose, ..
        // }) => {
        //     Err(ParserError {
        //         token: Some(token.clone()),
        //         reason: "unopened }".to_string(),
        //     })
        // }
        Some(Token {
            kind: TokenKind::BraceOpen,
            ..
        }) => Ok(Some(parse_object(iter)?)),
        Some(
            token @ Token {
                kind: TokenKind::BracketOpen,
                ..
            },
        ) => Ok(Some(parse_array(iter, token)?)),
        Some(Token {
            kind: TokenKind::String(s),
            ..
        }) => Ok(Some(Node::String(s.clone()))),
        Some(Token {
            kind: TokenKind::Number(s),
            ..
        }) => Ok(Some(number_to_node(s)?)),
        Some(
            token @ Token {
                kind: TokenKind::Value(_),
                ..
            },
        ) => Ok(Some(value_to_node(token.clone())?)),
        Some(token) => Err(ParserError {
            token: Some(token.clone()),
            reason: "unexpected token".to_string(),
        }),
        None => Ok(None),
    }
}

pub fn parse_array<'a, I: Iterator<Item = &'a Token>>(
    iter: &mut Peekable<I>,
    open: &Token,
) -> Result<Node, ParserError> {
    let mut array = Vec::<Node>::new();
    let mut last_is_value: bool = false;

    loop {
        match iter.peek() {
            Some(Token {
                kind: TokenKind::Colon,
                line,
                start,
                end,
            }) => {
                return Err(ParserError {
                    token: Some(Token {
                        line: *line,
                        start: *start,
                        end: *end,
                        kind: TokenKind::Colon,
                    }),
                    reason: "<key>:<value> is not an accepted element in arrays".to_string(),
                })
            }
            Some(Token {
                kind: TokenKind::Comma,
                line,
                start,
                end,
            }) => {
                if !last_is_value {
                    return Err(ParserError {
                        token: Some(Token {
                            line: *line,
                            start: *start,
                            end: *end,
                            kind: TokenKind::Comma,
                        }),
                        reason: "comma should be preceded by an element".to_string(),
                    });
                }
                last_is_value = false;
                iter.next();
            }
            Some(Token {
                kind: TokenKind::BracketClose,
                line,
                start,
                end,
            }) => {
                if !array.is_empty() && !last_is_value {
                    return Err(ParserError {
                        token: Some(Token {
                            line: *line,
                            start: *start,
                            end: *end,
                            kind: TokenKind::BracketClose,
                        }),
                        reason: "array should not end with a comma".to_string(),
                    });
                }

                iter.next();
                return Ok(Node::Array(array));
            }
            Some(Token {
                line,
                start,
                end,
                kind,
            }) => match parse(iter)? {
                Some(obj) => {
                    if last_is_value {
                        return Err(ParserError {
                            token: Some(Token {
                                line: *line,
                                start: *start,
                                end: *end,
                                kind: kind.clone(),
                            }),
                            reason: "elements in an array should be joined with a comma"
                                .to_string(),
                        });
                    }
                    last_is_value = true;
                    array.push(obj);
                }
                None => return Ok(Node::Array(array)),
            },
            None => {
                return Err(ParserError {
                    token: Some(open.clone()),
                    reason: "missing closing bracket".to_string(),
                })
            }
        }
    }
}

pub fn parse_object<'a, I: Iterator<Item = &'a Token>>(
    iter: &mut Peekable<I>,
) -> Result<Node, ParserError> {
    let mut mapping: HashMap<String, Node> = HashMap::new();

    loop {
        match iter.next() {
            Some(Token {
                kind: TokenKind::BraceClose,
                ..
            }) => {
                return Ok(Node::Object(mapping));
            }
            Some(
                token @ Token {
                    kind: TokenKind::String(k),
                    ..
                },
            ) => match iter.next() {
                Some(Token {
                    kind: TokenKind::Colon,
                    ..
                }) => match parse(iter)? {
                    Some(v) => {
                        mapping.insert(k.to_string(), v);
                    }
                    None => {
                        return Err(ParserError {
                            token: Some(token.clone()),
                            reason: "is missing value".to_string(),
                        });
                    }
                },
                _ => {
                    return Err(ParserError {
                        token: Some(token.clone()),
                        reason: "should be preceded by <colon><element>".to_string(),
                    });
                }
            },
            Some(
                token @ Token {
                    kind: TokenKind::Comma,
                    ..
                },
            ) => match iter.peek() {
                Some(_) => {
                    continue;
                }
                None => {
                    return Err(ParserError {
                        token: Some(token.clone()),
                        reason: "comma should be followed by a token".into(),
                    });
                }
            },
            None => {
                return Ok(Node::Object(mapping));
            }
            Some(t1) => {
                return Err(ParserError {
                    token: Some(t1.clone()),
                    reason: "unexpected token".to_string(),
                });
            }
        }
    }
}

impl Node {
    pub fn write<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), ParserError> {
        match self {
            Node::String(s) => {
                writer.write_all(&[b'"'])?;
                writer.write_all(s.as_bytes())?;
                writer.write_all(&[b'"'])?;
            }
            Node::Boolean(b) => {
                writer.write_all(b.to_string().as_bytes())?;
            }
            Node::Null => {
                writer.write_all(&[b'n', b'u', b'l', b'l'])?;
            }
            Node::Number(NumberNode::I64(i)) => {
                writer.write_all(i.to_string().as_bytes())?;
            }
            Node::Number(NumberNode::F64(f)) => {
                writer.write_all(f.to_string().as_bytes())?;
            }
            Node::Array(nodes) => {
                writer.write_all(&[b'['])?;
                let n = nodes.len();
                for (i, node) in nodes.iter().enumerate() {
                    node.write(writer)?;
                    if i != n - 1 {
                        writer.write_all(&[b','])?;
                    }
                }
                writer.write_all(&[b']'])?;
            }
            Node::Object(mapping) => {
                writer.write_all(&[b'{'])?;
                let n = mapping.len();
                for (i, (k, v)) in mapping.iter().enumerate() {
                    writer.write_all(&[b'"'])?;
                    writer.write_all(k.as_bytes())?;
                    writer.write_all(&[b'"', b':'])?;
                    v.write(writer)?;

                    if i != n - 1 {
                        writer.write_all(&[b','])?;
                    }
                }
                writer.write_all(&[b'}'])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {

    use crate::error::ParserError;
    use std::io::Cursor;

    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case(r#"true"#, Node::Boolean(true))]
    #[case(r#"false"#, Node::Boolean(false))]
    #[case(r#"null"#, Node::Null)]
    fn test_take_value(#[case] input: &str, #[case] expected: Node) {
        let tokens = tokenize(Cursor::new(input).lines()).unwrap();
        let mut iter = tokens.iter().peekable();
        let node = parse(&mut iter).unwrap().unwrap();
        assert_eq!(node, expected);
    }

    #[rstest]
    #[case(
        r#"test""#,
        Token {
            line: 0,
            start: 0,
            end: 4,
            kind: TokenKind::String("test".to_string()),
        },
    )]
    #[case(
        r#"\"""#,
        Token {
            line: 0,
            start: 0,
            end: 1,
            kind: TokenKind::String("\"".to_string()),
        },
    )]
    #[case(
        r#"\u1f44d""#,
        Token {
            line: 0,
            start: 0,
            end: 4,
            kind: TokenKind::String("👍".to_string()),
        },
    )]
    #[case(
        r#"\u1f44d\"\u1f625""#,
        Token {
            line: 0,
            start: 0,
            end: 9,
            kind: TokenKind::String("👍\"😥".to_string()),
        },
    )]
    fn test_take_string(#[case] input: &str, #[case] expected: Token) {
        let mut iter = input.char_indices().peekable();
        let node = take_string(&mut iter, 0, 0).unwrap();
        assert_eq!(node, expected);
    }

    #[rstest]
    #[case(
        r#"{
            "index": 0
        }"#,
        vec![
            Token {line: 0, start: 0, end: 1, kind: TokenKind::BraceOpen},
            Token {line: 1, start: 12, end: 17, kind: TokenKind::String("index".to_string())},
            Token {line: 1, start: 19, end: 20, kind: TokenKind::Colon},
            Token {line: 1, start: 21, end: 21, kind: TokenKind::Number("0".to_string())},
            Token {line: 2, start: 8, end: 9, kind: TokenKind::BraceClose},
        ]
    )]
    #[case(
        r#"{
            "id": "647ceaf3657eade56f8224eb",
            "index": 0
        }"#,
        vec![
            Token {line: 0, start: 0, end: 1, kind: TokenKind::BraceOpen},
            Token {line: 1, start: 12, end: 14, kind: TokenKind::String("id".to_string())},
            Token {line: 1, start: 16, end: 17, kind: TokenKind::Colon},
            Token {line: 1, start: 18, end: 42, kind: TokenKind::String("647ceaf3657eade56f8224eb".to_string())},
            Token {line: 1, start: 44, end: 45, kind: TokenKind::Comma},
            Token {line: 2, start: 12, end: 17, kind: TokenKind::String("index".to_string())},
            Token {line: 2, start: 19, end: 20, kind: TokenKind::Colon},
            Token {line: 2, start: 21, end: 21, kind: TokenKind::Number("0".to_string())},
            Token {line: 3, start: 8, end: 9, kind: TokenKind::BraceClose},
        ]
    )]
    #[case(
        r#"{
            "id": "647ceaf3657eade56f8224eb",
            "index": 0,
            "something": [],
            "boolean": true,
            "nullValue": null
        }"#,
        vec![
            Token {line: 0, start: 0, end: 1, kind: TokenKind::BraceOpen},

            Token {line: 1, start: 12, end: 14, kind: TokenKind::String("id".to_string())},
            Token {line: 1, start: 16, end: 17, kind: TokenKind::Colon},
            Token {line: 1, start: 18, end: 42, kind: TokenKind::String("647ceaf3657eade56f8224eb".to_string())},
            Token {line: 1, start: 44, end: 45, kind: TokenKind::Comma},

            Token {line: 2, start: 12, end: 17, kind: TokenKind::String("index".to_string())},
            Token {line: 2, start: 19, end: 20, kind: TokenKind::Colon},
            Token {line: 2, start: 21, end: 21, kind: TokenKind::Number("0".to_string())},
            Token {line: 2, start: 22, end: 23, kind: TokenKind::Comma},

            Token {line: 3, start: 12, end: 21, kind: TokenKind::String("something".to_string())},
            Token {line: 3, start: 23, end: 24, kind: TokenKind::Colon},
            Token {line: 3, start: 25, end: 26, kind: TokenKind::BracketOpen},
            Token {line: 3, start: 26, end: 27, kind: TokenKind::BracketClose},
            Token {line: 3, start: 27, end: 28, kind: TokenKind::Comma},

            Token {line: 4, start: 12, end: 19, kind: TokenKind::String("boolean".to_string())},
            Token {line: 4, start: 21, end: 22, kind: TokenKind::Colon},
            Token {line: 4, start: 24, end: 27, kind: TokenKind::Value("true".to_string())},
            Token {line: 4, start: 27, end: 28, kind: TokenKind::Comma},

            Token {line: 5, start: 12, end: 21, kind: TokenKind::String("nullValue".to_string())},
            Token {line: 5, start: 23, end: 24, kind: TokenKind::Colon},
            Token {line: 5, start: 26, end: 29, kind: TokenKind::Value("null".to_string())},

            Token {line: 6, start: 8, end: 9, kind: TokenKind::BraceClose},
        ]
    )]
    fn test_tokenize(#[case] input: &str, #[case] expected: Vec<Token>) {
        assert_eq!(tokenize(Cursor::new(input).lines()).unwrap(), expected);
    }

    #[rstest]
    #[case(
        r#"{"outer": ["1", "2", "3"]}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Array(vec![
                Node::String("1".to_string()),
                Node::String("2".to_string()),
                Node::String("3".to_string()),
            ])),
        ]))
    )]
    #[case(
        r#"{"outer": [null, true, "3"]}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Array(vec![
                Node::Null,
                Node::Boolean(true),
                Node::String("3".to_string()),
            ])),
        ]))
    )]
    #[case(
        r#"{"outer": [{}]}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Array(vec![
                Node::Object(HashMap::new()),
            ])),
        ]))
    )]
    #[case(
        r#"{"outer": [{}, {}, {}]}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Array(vec![
                Node::Object(HashMap::new()),
                Node::Object(HashMap::new()),
                Node::Object(HashMap::new()),
            ])),
        ]))
    )]
    #[case(
        r#"{"outer": [{"index": 0}, {}, {}]}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Array(vec![
                Node::Object(HashMap::from_iter(vec![
                    ("index".to_string(), Node::Number(NumberNode::I64(0))),
                ])),
                Node::Object(HashMap::new()),
                Node::Object(HashMap::new()),
            ])),
        ]))
    )]
    #[case(
        r#"[{"index": 0}, {"value": 1}]"#,
        Node::Array(vec![
            Node::Object(HashMap::from_iter(vec![
                ("index".to_string(), Node::Number(NumberNode::I64(0))),
            ])),
            Node::Object(HashMap::from_iter(vec![
                ("value".to_string(), Node::Number(NumberNode::I64(1))),
            ]))
        ])
    )]
    #[case(
        r#"{"index":0}"#,
        Node::Object(HashMap::from_iter(vec![
            ("index".to_string(), Node::Number(NumberNode::I64(0))),
        ]))
    )]
    fn test_array_parsing(#[case] input: &str, #[case] expected: Node) {
        let tokens = tokenize(Cursor::new(input).lines()).unwrap();
        let mut iter = tokens.iter().peekable();

        let res = parse(&mut iter).unwrap().unwrap();
        assert_eq!(res, expected);
    }

    #[rstest]
    #[case(
        r#"{"outer": {"inner": "value"}, "random": "value"}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Object(HashMap::from_iter(vec![
                ("inner".to_string(), Node::String("value".to_string()))
            ]))),
            ("random".to_string(), Node::String("value".to_string()))
        ])),
    )]
    #[case(
        r#"{"outer": {"inner": "\u1f911"}, "random": "value"}"#,
        Node::Object(HashMap::from_iter(vec![
            ("outer".to_string(), Node::Object(HashMap::from_iter(vec![
                ("inner".to_string(), Node::String("🤑".to_string()))
            ]))),
            ("random".to_string(), Node::String("value".to_string()))
        ])),
    )]
    fn test_object_parsing(#[case] input: &str, #[case] expected: Node) {
        let tokens = tokenize(Cursor::new(input).lines()).unwrap();
        let mut iter = tokens.iter().peekable();

        let res = parse(&mut iter).unwrap().unwrap();
        assert_eq!(res, expected);
    }

    #[rstest]
    #[case("0", Node::Number(NumberNode::I64(0)))]
    #[case(
        "-1", Node::Number(NumberNode::I64(-1)),
    )]
    #[case("123", Node::Number(NumberNode::I64(123)))]
    #[case("123.456", Node::Number(NumberNode::F64(123.456)))]
    #[case("123.456e52", Node::Number(NumberNode::F64(123.456e52)))]
    #[case("123.456e-2", Node::Number(NumberNode::F64(123.456e-2)))]
    fn test_number_parsing(#[case] input: &str, #[case] expected: Node) {
        let node = number_to_node(input).unwrap();
        assert_eq!(node, expected);
    }

    #[rstest]
    #[case(
        Node::Object(HashMap::from_iter(vec![
            (
                "outer".to_string(),
                Node::Object(HashMap::from_iter(vec![(
                    "inner".to_string(),
                    Node::String("🤑".to_string()),
                )])),
            ),
            ("random".to_string(), Node::String("value".to_string())),
        ])),
    )]
    fn test_write(#[case] node: Node) {
        let mut buf = BufWriter::new(Vec::new());
        node.write(&mut buf).unwrap();
        let bytes = buf.into_inner().unwrap();

        let tokens = tokenize(Cursor::new(bytes).lines()).unwrap();
        let mut iter = tokens.iter().peekable();

        let res = parse(&mut iter).unwrap().unwrap();
        assert_eq!(res, node);
    }

    #[rstest]
    #[case(
        r#"0.4e00669999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999969999999006"#,
        Ok(Some(Node::Number(NumberNode::F64(f64::INFINITY))))
    )]
    #[case(
        r#"-1e+9999"#,
        Ok(Some(Node::Number(NumberNode::F64(-f64::INFINITY))))
    )]
    #[case(r#"1.5e+9999"#, Ok(Some(Node::Number(NumberNode::F64(f64::INFINITY)))))]
    #[case(
        r#"-123123e100000"#,
        Ok(Some(Node::Number(NumberNode::F64(-f64::INFINITY))))
    )]
    #[case(
        r#"123123e100000"#,
        Ok(Some(Node::Number(NumberNode::F64(f64::INFINITY))))
    )]
    #[case(r#"123e-10000000"#, Ok(Some(Node::Number(NumberNode::F64(0.0)))))]
    #[case(
        r#"-123123123123123123123123123123"#,
        Err(ParserError { token: None, reason: "number too small to fit in target type".to_string() })
    )]
    #[case(
        r#"100000000000000000000"#,
        Err(ParserError { token: None, reason: "number too large to fit in target type".to_string() })
    )]
    #[case(
        r#"-237462374673276894279832749832423479823246327846"#,
        Err(ParserError { token: None, reason: "number too small to fit in target type".to_string() })
    )]
    #[case(
        r#"[1 true]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 4, end: 7, kind: TokenKind::Value("true".to_string())}), 
            reason: "elements in an array should be joined with a comma".to_string() })
    )]
    #[case(
        r#"[1, ]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 4, end: 5, kind: TokenKind::BracketClose}), 
            reason: "array should not end with a comma".to_string() })
    )]
    #[case(
        r#"["": 1]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 3, end: 4, kind: TokenKind::Colon}), 
            reason: "<key>:<value> is not an accepted element in arrays".to_string() }) 
    )]
    #[case(
        r#"[""],"#,
        Ok(Some(Node::Array(vec![Node::String("".to_string())])))
    )]
    #[case(
        r#"[,1]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 1, end: 2, kind: TokenKind::Comma}), 
            reason: "comma should be preceded by an element".to_string() })
    )]
    #[case(
        r#"[1,,2]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 3, end: 4, kind: TokenKind::Comma}), 
            reason: "comma should be preceded by an element".to_string() })
    )]
    #[case(
        r#"["x",,]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 5, end: 6, kind: TokenKind::Comma}), 
            reason: "comma should be preceded by an element".to_string() })
    )]
    #[case(
        r#"["",]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 4, end: 5, kind: TokenKind::BracketClose}), 
            reason: "array should not end with a comma".to_string() })
    )]
    #[case(
        r#"[1,]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 3, end: 4, kind: TokenKind::BracketClose}), 
            reason: "array should not end with a comma".to_string() })
    )]
    #[case(
        r#"["",]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 4, end: 5, kind: TokenKind::BracketClose}), 
            reason: "array should not end with a comma".to_string() })
    )]
    #[case(
        r#"[,]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 1, end: 2, kind: TokenKind::Comma}), 
            reason: "comma should be preceded by an element".to_string() })
    )]
    #[case(
        r#"[   , ""]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 4, end: 5, kind: TokenKind::Comma}), 
            reason: "comma should be preceded by an element".to_string() })
    )]
    #[case(
        r#"{"outer": ["x"]]}"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 15, end: 16, kind: TokenKind::BracketClose}), 
            reason: "unexpected token".to_string() })
    )]
    #[case(
        r#"["x""#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 0, end: 1, kind: TokenKind::BracketOpen}), 
            reason: "missing closing bracket".to_string() })
    )]
    #[case(
        r#"[x"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 1, end: 1, kind: TokenKind::Value("x".to_string())}), 
            reason: "invalid value node".to_string() })
    )]
    #[case(
        r#"[-]"#,
        Err(ParserError { 
            token: None,
            reason: "invalid digit found in string".to_string() })
    )]
    #[case(
        r#"[3[4]]"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 2, end: 3, kind: TokenKind::BracketOpen}), 
            reason: "elements in an array should be joined with a comma".to_string() })
    )]
    #[case(
        r#"["a",
        4
        ,1,"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 0, end: 1, kind: TokenKind::BracketOpen}), 
            reason: "missing closing bracket".to_string() })
    )]
    #[case(
        r#"[]"#,
        Ok(Some(Node::Array(vec![])))
    )]
    #[case(
        r#"{"outer": }"#,
        Err(ParserError { 
            token: Some(Token{line: 0, start: 10, end: 11, kind: TokenKind::BraceClose}), 
            reason: "unexpected token".to_string() })
    )]
    fn test_edge_cases(#[case] input: &str, #[case] expected: Result<Option<Node>, ParserError>) {
        let tokens = tokenize(Cursor::new(input).lines()).unwrap();
        let mut iter = tokens.iter().peekable();

        let res = parse(&mut iter);
        assert_eq!(res, expected);
    }

    #[rstest]
    #[case(
        r#"{"\uDFAA":0}"#,
       ParserError { token: Some(Token{line: 0, start: 2, end: 6, kind: TokenKind::BackSlash}), reason: "invalid unicode character".to_string() }
    )]
    #[case(
        r#"["\uDADA"]"#,
        ParserError { token: Some(Token{line: 0, start: 2, end: 6, kind: TokenKind::BackSlash}), reason: "invalid unicode character".to_string() }
    )]
    #[case(
        r#"["\uD888\u1234"]""#,
        ParserError { token: Some(Token{line: 0, start: 2, end: 6, kind: TokenKind::BackSlash}), reason: "invalid unicode character".to_string() }
    )]
    #[case(
        r#"{"\uAB"}"#,
        ParserError { 
            token: Some(Token{line: 0, start: 2, end: 4, kind: TokenKind::BackSlash}), 
            reason: "unicode character should contain 4..6 hex digits".to_string() 
        }
    )]
    #[case(
        r#"{"\A"}"#,
        ParserError { 
            token: Some(Token{line: 0, start: 2, end: 2, kind: TokenKind::BackSlash}), 
            reason: "unclosed escape character \\".to_string() 
        }
    )]
    #[case(
        r#"{"\"#,
        ParserError { 
            token: Some(Token{line: 0, start: 1, end: 1, kind: TokenKind::BackSlash}), 
            reason: "unexpected EOF, unclosed escape character \\".to_string() 
        }
    )]
    #[case(
        r#""abc"#,
        ParserError { 
            token: Some(Token{line: 0, start: 0, end: 3, kind: TokenKind::Quote}), 
            reason: "unclosed quote \"".to_string() 
        }
    )]
    fn test_tokenize_error(#[case] input: &str, #[case] expected: ParserError) {
        let tokens = tokenize(Cursor::new(input).lines());
        assert!(tokens.is_err());
        assert_eq!(tokens.unwrap_err(), expected);
    }
}
