use bytes::buf::BlockBuf;
use bytes::{Buf, MutBuf};
use futures::{Async, Poll};
use std::io;
use tokio_core::io::{FramedIo, Io};
use tokio_proto::{pipeline, Framed, Parse, Serialize};
use super::Frame;

pub struct DelimiterTransport<T, D: Delimiter> {
    inner: Framed<T, Parser<D>, Serializer<D>>,
}

impl<T, D> DelimiterTransport<T, D>
    where T: Io,
          D: Delimiter + Clone
{
    pub fn new(upstream: T, delimiter: D, rd: BlockBuf, wr: BlockBuf) -> Self {
        DelimiterTransport {
            inner: Framed::new(upstream,
                               Parser { delimiter: delimiter.clone() },
                               Serializer { delimiter: delimiter },
                               rd,
                               wr),
        }
    }
}

impl<T, D> FramedIo for DelimiterTransport<T, D>
    where T: Io,
          D: Delimiter
{
    type In = Frame;
    type Out = Frame;

    fn poll_read(&mut self) -> Async<()> {
        self.inner.poll_read()
    }

    fn read(&mut self) -> Poll<Self::Out, io::Error> {
        self.inner.read()
    }

    fn poll_write(&mut self) -> Async<()> {
        self.inner.poll_write()
    }

    fn write(&mut self, req: Self::In) -> Poll<(), io::Error> {
        self.inner.write(req)
    }

    fn flush(&mut self) -> Poll<(), io::Error> {
        self.inner.flush()
    }
}

pub struct Parser<D: Delimiter> {
    pub delimiter: D,
}

impl<D: Delimiter> Parse for Parser<D> {
    type Out = Frame;

    fn parse(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
        match self.delimiter.pop_buf(buf) {
            Ok(Some(frame)) => Some(pipeline::Frame::Message(frame)),
            Ok(None) => None,
            Err(e) => Some(pipeline::Frame::Error(e)),
        }
    }
}

pub struct Serializer<D: Delimiter> {
    pub delimiter: D,
}

impl<D: Delimiter> Serialize for Serializer<D> {
    type In = Frame;

    fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
        match frame {
            pipeline::Frame::Message(v) => {
                buf.write_slice(v.as_slice());
                self.delimiter.write_deliimiter(buf);
            }
            other => panic!("cannot serialize {:?}", other),
        }
    }
}

pub trait Delimiter {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error>;
    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B);
}

impl Delimiter for u8 {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        buf.compact();
        debug_assert!(buf.is_compact());

        let pos = buf.bytes()
            .and_then(|bs| bs.into_iter().position(|x| x == self));

        Ok(pos.and_then(move |pos| {
            let bs = buf.shift(pos + 1);
            bs.buf()
                .bytes()
                .split_last()
                .map(|(_, frame)| frame.into())
        }))
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(&[*self]);
    }
}

#[test]
fn test_delimiter_u8() {
    let mut buf = BlockBuf::default();
    buf.write_slice(&[1, 0, 2, 0, 0]);

    let delimiter = 0;

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![1u8]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![2u8]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    delimiter.write_deliimiter(&mut buf);
    buf.compact();
    assert_eq!(buf.bytes().unwrap(), &[0u8]);
}

impl Delimiter for char {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        buf.compact();
        debug_assert!(buf.is_compact());

        let pos = match buf.bytes().map(|bs| ::std::str::from_utf8(bs)) {
            None => return Ok(None),
            Some(Err(err)) => {
                return Err(io::Error::new(io::ErrorKind::Other, format!("{}", err)));
            }
            Some(Ok(s)) => s.char_indices().find(|&(_, c)| c == *self).map(|(i, _)| i),
        };

        Ok(pos.map(move |pos| {
            let bs = buf.shift(pos + self.len_utf8());
            bs.buf()
                .bytes()
                .split_at(pos)
                .0
                .to_vec()
        }))
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        // TODO: use `char::encode_utf8` once it is stabilized

        // from rust/src/libcore:

        // UTF-8 ranges and tags for encoding characters
        const TAG_CONT: u8 = 0b1000_0000;
        const TAG_TWO_B: u8 = 0b1100_0000;
        const TAG_THREE_B: u8 = 0b1110_0000;
        const TAG_FOUR_B: u8 = 0b1111_0000;
        const MAX_ONE_B: u32 = 0x80;
        const MAX_TWO_B: u32 = 0x800;
        const MAX_THREE_B: u32 = 0x10000;

        let code = *self as u32;
        if code < MAX_ONE_B {
            buf.write_slice(&[code as u8]);
        } else if code < MAX_TWO_B {
            buf.write_slice(&[(code >> 6 & 0x1F) as u8 | TAG_TWO_B,
                              (code & 0x3F) as u8 | TAG_CONT]);
        } else if code < MAX_THREE_B {
            buf.write_slice(&[(code >> 12 & 0x0F) as u8 | TAG_THREE_B,
                              (code >> 6 & 0x3F) as u8 | TAG_CONT,
                              (code & 0x3F) as u8 | TAG_CONT]);
        } else {
            buf.write_slice(&[(code >> 18 & 0x07) as u8 | TAG_FOUR_B,
                              (code >> 12 & 0x3F) as u8 | TAG_CONT,
                              (code >> 6 & 0x3F) as u8 | TAG_CONT,
                              (code & 0x3F) as u8 | TAG_CONT]);
        }
    }
}

#[test]
fn test_delimiter_char() {
    let mut buf = BlockBuf::default();
    buf.write_slice("あめ、つち、、".as_bytes());

    let delimiter = '、';

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    delimiter.write_deliimiter(&mut buf);
    buf.compact();
    assert_eq!(buf.bytes().unwrap(), "、".as_bytes());
}

#[derive(Debug)]
pub struct LineDelimiter;
// TODO: config

// NOTE: parses '\r', '\n', '\r\n', writes '\n'
impl Delimiter for LineDelimiter {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        buf.compact();
        debug_assert!(buf.is_compact());

        let ret = match buf.bytes() {
                None => return Ok(None),
                Some(bs) => bs.into_iter().position(|&c| c == b'\r' || c == b'\n'),
            }
            .and_then(move |pos| {
                let shift = buf.shift(pos + 1);
                let bs = shift.buf();
                let bs = bs.bytes();

                // shift another 1-byte if line breaker is "\r\n"
                if !buf.is_empty() && bs.last() == Some(&b'\r') && buf.buf().peek_u8() == Some(b'\n') {
                    buf.shift(1);
                }

                bs.split_last()
                    .map(|(_, frame)| frame.into())
            });

        Ok(ret)
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(b"\n");
    }
}

#[test]
fn test_delimiter_line() {
    let mut buf = BlockBuf::default();
    buf.write_slice("あめ\nつち\r\n\n".as_bytes());

    let delimiter = LineDelimiter;

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    delimiter.write_deliimiter(&mut buf);
    buf.compact();
    assert_eq!(buf.bytes().unwrap(), b"\n");
}

impl<'a> Delimiter for &'a [u8] {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        if buf.len() < self.len() {
            return Ok(None);
        }

        buf.compact();
        debug_assert!(buf.is_compact());

        let start = buf.bytes()
            .and_then(|b| {
                debug_assert!(buf.len() >= self.len());
                for i in 0 .. b.len() - self.len() + 1 {
                    if b[i..].starts_with(self) {
                        return Some(i);
                    }
                }

                None
            });

        Ok(start.map(move |start| {
            let b = buf.shift(start + self.len());
            (&b.buf().bytes()[..start]).into()
        }))
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(self);
    }
}

impl Delimiter for Vec<u8> {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        self.as_slice().pop_buf(buf)
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(self.as_slice());
    }
}

#[test]
fn test_delimiter_vec() {
    let mut buf = BlockBuf::default();
    buf.write_slice("あめ#\0#つち#\0##\0#".as_bytes());

    let delimiter = &b"#\0#"[..].to_vec();

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    delimiter.write_deliimiter(&mut buf);
    buf.compact();
    assert_eq!(buf.bytes().unwrap(), b"#\0#");
}


impl<'a> Delimiter for &'a str {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(self.as_bytes());
    }
}

impl Delimiter for String {
    fn pop_buf(&self, buf: &mut BlockBuf) -> Result<Option<Vec<u8>>, io::Error> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_deliimiter<B: MutBuf>(&self, buf: &mut B) {
        buf.write_slice(self.as_bytes());
    }
}

#[test]
fn test_delimiter_string() {
    let mut buf = BlockBuf::default();
    buf.write_slice("あめ・\nつち・\n・\n".as_bytes());

    let delimiter = "・\n";

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    delimiter.write_deliimiter(&mut buf);
    buf.compact();
    assert_eq!(buf.bytes().unwrap(), "・\n".as_bytes());
}
