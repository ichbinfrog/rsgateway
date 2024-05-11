use std::{cmp, collections::VecDeque, fmt::Debug, io::{BufWriter, Write}, slice::Windows};

pub struct Match {
    offset: usize,
    length: usize,
    value: u8,
}

impl Debug for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.offset, self.length, self.value as char,
        )
    }
}

fn find(haystack: &Vec<u8>, needle: &[u8]) -> Option<usize> {
    for i in (0..haystack.len() - needle.len() + 1).rev() {
        if haystack[i..i + needle.len()] == *needle {
            return Some(i);
        }
    }
    None
}

#[derive(Debug)]
pub struct Buffer<'a> {
    search_buffer_length: usize,
    lookahead_buffer_length: usize,

    search_buffer: Vec<u8>,
    lookahead_buffer: Windows<'a, u8>,
    input: &'a [u8],
}

impl<'a> Buffer<'a> {
    fn new(input: &'a [u8], search_buffer_length: usize, lookahead_buffer_length: usize) -> Self {
        Self {
            search_buffer_length,
            lookahead_buffer_length,
            input,
            search_buffer: Vec::with_capacity(search_buffer_length),
            lookahead_buffer: input.windows(lookahead_buffer_length),
        }
    }

    fn shift_search_buffer(&mut self, new: u8) {
        if self.search_buffer.len() == self.search_buffer_length {
            self.search_buffer.remove(0);
        }
        self.search_buffer.push(new);
    }

    fn write<W>(&self, writer: &mut BufWriter<W>, m: Match) 
    where W: Write
    {
        writer.write(&[
            m.offset as u8,
            m.length as u8,
            m.value as u8,
        ]).unwrap();
    }

    fn compress<W>(&mut self, writer: &mut BufWriter<W>) 
    where W: Write
    {
        let mut hop = 0;

        while let Some(lookahead) = self.lookahead_buffer.next() {
            if hop > 0 {
                hop -= 1;
                self.shift_search_buffer(lookahead[0]);
                continue;
            }

            match self.find_longest_match(&self.search_buffer, lookahead) {
                Some(m) => {
                    if m.length > 0 {
                        hop += m.length;
                    }
                    self.write(writer, m);
                }
                None => {}
            }

            self.shift_search_buffer(lookahead[0]);
        }

        for i in (0..self.lookahead_buffer_length-1).rev() {
            let lookahead = &self.input[self.input.len() - 1 - i..];
            if hop > 0 {
                hop -= 1;
                self.shift_search_buffer(lookahead[0]);
                continue;
            }

            match self.find_longest_match(&self.search_buffer, lookahead) {
                Some(m) => {
                    if m.length > 0 {
                        hop += m.length;
                    }
                    self.write(writer, m);
                }
                None => {}
            }
            self.shift_search_buffer(lookahead[0]);
        }

        writer.flush();
    }

    fn find_longest_match(
        &self,
        search_buffer: &Vec<u8>,
        lookahead_buffer: &[u8],
    ) -> Option<Match> {
        let ns = search_buffer.len();
        let nl = lookahead_buffer.len();

        if ns == 0 {
            return Some(Match {
                offset: 0,
                length: 0,
                value: lookahead_buffer[0],
            });
        }

        for n in (1..std::cmp::min(ns, nl)).rev() {
            let index = find(search_buffer, &lookahead_buffer[..n]);
            match index {
                Some(x) => {
                    return Some(Match {
                        offset: ns - x,
                        length: n,
                        value: lookahead_buffer[n],
                    });
                }
                None => {}
            }
        }

        Some(Match {
            offset: 0,
            length: 0,
            value: lookahead_buffer[0],
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::{fs::File, io::BufWriter};

    use super::{find, Buffer};

    #[test]
    fn test_find() {
        let haystack: Vec<u8> = vec![97, 98, 99, 98, 98, 99, 98];
        let needle: &[u8] = &[99, 98, 98];
        println!("{:?}", find(&haystack, &needle));
    }

    #[test]
    fn test_impl() {
        let mut buf = Buffer::new("Hello friends, Hello world".as_bytes(), 15, 5);

        let f = File::create("tests/les_miserables_compressed").unwrap();
        let mut writer = BufWriter::new(f);

        buf.compress(&mut writer);
    }
}
