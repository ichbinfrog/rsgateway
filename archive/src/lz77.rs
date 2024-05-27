use std::{
    error::Error,
    fmt::Debug,
    io::{BufReader, BufWriter, Read, Write},
    slice::Windows,
};

pub struct Match {
    offset: usize,
    length: usize,
    value: u8,
}

#[derive(Debug)]
pub struct SearchBuffer {
    data: Vec<u8>,
    size: usize,
}

#[inline]
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }

    (0..haystack.len() - needle.len() + 1)
        .rev()
        .find(|&i| haystack[i..i + needle.len()] == *needle)
}

impl SearchBuffer {
    fn new(n: usize) -> Self {
        Self {
            data: Vec::with_capacity(n),
            size: n,
        }
    }

    #[inline]
    fn insert(&mut self, new: u8) {
        if self.data.len() == self.size {
            self.data.remove(0);
        }
        self.data.push(new);
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl From<[u8; 3]> for Match {
    #[inline]
    fn from(value: [u8; 3]) -> Self {
        Self {
            offset: (((value[0] & 0xF) << 4) | ((value[1] & !0xF) >> 4)) as usize,
            length: (value[1] & 0xF) as usize,
            value: value[2],
        }
    }
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

#[derive(Debug)]
pub struct Buffer<'a> {
    search_buffer_length: usize,
    lookahead_buffer_length: usize,

    lookahead_buffer: Windows<'a, u8>,
    input: &'a [u8],
}

impl<'a> Buffer<'a> {
    pub fn new(
        input: &'a [u8],
        search_buffer_length: usize,
        lookahead_buffer_length: usize,
    ) -> Self {
        Self {
            search_buffer_length,
            lookahead_buffer_length,
            input,
            lookahead_buffer: input.windows(lookahead_buffer_length),
        }
    }

    fn decompress<R, W>(
        &mut self,
        reader: &mut BufReader<R>,
        writer: &mut BufWriter<W>,
    ) -> Result<(), Box<dyn Error>>
    where
        W: Write,
        R: Read,
    {
        let mut search_buffer = SearchBuffer::new(self.search_buffer_length);
        loop {
            let mut buf = [0u8; 3];

            match reader.read(&mut buf) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    let m = Match::from(buf);
                    if m.offset == 0 && m.length == 0 {
                        search_buffer.insert(m.value);
                        writer.write_all(&[m.value])?;
                        continue;
                    }

                    let cur_buffer_length = search_buffer.len();

                    let start = cur_buffer_length - m.offset;
                    let end = cur_buffer_length - m.offset + m.length;

                    writer.write_all(&search_buffer.data[start..end])?;
                    for i in start..end {
                        search_buffer.insert(search_buffer.data[i]);
                    }
                    search_buffer.insert(m.value);
                    writer.write_all(&[m.value])?;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    #[inline]
    fn write<W>(&self, writer: &mut BufWriter<W>, m: Match)
    where
        W: Write,
    {
        writer
            .write_all(&[
                ((m.offset & !0xF) >> 4) as u8,
                ((m.offset & 0xF) << 4) as u8 | (m.length & 0xF) as u8,
                m.value,
            ])
            .unwrap();
    }

    #[inline]
    pub fn compress<W>(&mut self, writer: &mut BufWriter<W>) -> Result<(), Box<dyn Error>>
    where
        W: Write,
    {
        let mut hop = 0;
        let mut search_buffer = SearchBuffer::new(self.search_buffer_length);

        while let Some(lookahead) = self.lookahead_buffer.next() {
            if hop > 0 {
                hop -= 1;
                search_buffer.insert(lookahead[0]);
                continue;
            }

            match self.find_longest_match(&search_buffer.data, lookahead) {
                Some(m) => {
                    if m.length > 0 {
                        hop += m.length;
                    }
                    self.write(writer, m);
                }
                None => {}
            }

            search_buffer.insert(lookahead[0]);
        }

        for i in (0..self.lookahead_buffer_length - 1).rev() {
            let lookahead = &self.input[self.input.len() - 1 - i..];
            if hop > 0 {
                hop -= 1;
                search_buffer.insert(lookahead[0]);
                continue;
            }

            match self.find_longest_match(&search_buffer.data, lookahead) {
                Some(m) => {
                    if m.length > 0 {
                        hop += m.length;
                    }
                    self.write(writer, m);
                }
                None => {}
            }
            search_buffer.insert(lookahead[0]);
        }

        writer.flush()?;
        Ok(())
    }

    #[inline]
    fn find_longest_match(&self, search_buffer: &[u8], lookahead_buffer: &[u8]) -> Option<Match> {
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
            if let Some(x) = index {
                return Some(Match {
                    offset: ns - x,
                    length: n,
                    value: lookahead_buffer[n],
                });
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

    use super::find;
    use rstest::*;

    #[rstest]
    #[case(vec![0, 1, 2, 1, 1, 2, 1], &[2, 1, 1], Some(2))]
    #[case(vec![2, 2, 2], &[2, 2, 2], Some(0))]
    #[case(vec![0, 1, 2, 1, 1, 2, 1], &[2, 2, 2], None)]
    #[case(vec![], &[2, 2, 2], None)]
    #[case(vec![2], &[2, 2, 2], None)]
    fn test_vector_find(
        #[case] haystack: Vec<u8>,
        #[case] needle: &[u8],
        #[case] index: Option<usize>,
    ) {
        assert_eq!(find(&haystack, needle), index);
    }

    // #[test]
    // fn test_impl() {
    //     let filename = "benches/testdata/les_miserables.txt";
    //     let mut input = File::open(filename).unwrap();
    //     let metadata = fs::metadata(filename).unwrap();
    //     let mut input_buffer: Vec<u8> = vec![0; metadata.len() as usize];
    //     input.read_exact(&mut input_buffer).unwrap();

    //     let mut buf: Buffer<'_> = Buffer::new(&input_buffer, 4095, 15);

    //     let f = File::create("benches/testdata/les_miserables_compressed").unwrap();
    //     let mut writer = BufWriter::new(f);

    //     buf.compress(&mut writer).unwrap();
    // }

    // #[test]
    // fn test_read() {
    //     let filename = "benches/testdata/les_miserables_compressed";
    //     let input = File::open(filename).unwrap();
    //     let mut buf: Buffer<'_> = Buffer::new(&[], 4095, 15);

    //     let mut reader = BufReader::new(input);

    //     let output = File::create("tests/les_miserables_decompressed").unwrap();
    //     let mut writer = BufWriter::new(output);
    //     println!("{:?}", buf.decompress(&mut reader, &mut writer));
    // }
}
