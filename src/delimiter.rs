use tokio_core::io::{Codec, Io, EasyBuf, Framed};
use tokio_proto::pipeline::{ServerProto, ClientProto};
use std::io;

/// A protocol such that frames are separated with specified delimiters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelimiterProto<D>(D);

impl<D: Delimiter> DelimiterProto<D> {
    /// Creates a `DelimiterProto` from the specified delimiter.
    pub fn new(delimiter: D) -> Self {
        DelimiterProto(delimiter)
    }
}

impl<T, D> ServerProto<T> for DelimiterProto<D>
    where T: Io + 'static,
          D: Delimiter + Clone + 'static
{
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Error = io::Error;
    type Transport = Framed<T, DelimiterCodec<D>>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(DelimiterCodec(self.0.clone())))
    }
}

impl<T: Io, D> ClientProto<T> for DelimiterProto<D>
    where T: Io + 'static,
          D: Delimiter + Clone + 'static
{
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Error = io::Error;
    type Transport = Framed<T, DelimiterCodec<D>>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(DelimiterCodec(self.0.clone())))
    }
}

#[derive(Debug, Clone)]
pub struct DelimiterCodec<D>(D);

impl<D> DelimiterCodec<D> {
    pub fn new(delimiter: D) -> DelimiterCodec<D> {
        DelimiterCodec(delimiter)
    }
}

impl<D> Codec for DelimiterCodec<D>
    where D: Delimiter + Clone
{
    type In = Vec<u8>;
    type Out = Vec<u8>;

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        self.0.pop_buf(buf)
    }

    #[inline]
    fn encode(&mut self, item: Vec<u8>, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.extend_from_slice(item.as_slice());
        self.0.write_delimiter(buf);
        Ok(())
    }
}

/// A delimiter.
pub trait Delimiter {
    /// Removes elements from buffer including next occurence of this delimiter,
    /// and returns the removed part except the delimiter.
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>>;
    /// Appends this delimiter to the buffer.
    fn write_delimiter(&self, buf: &mut Vec<u8>);
}

impl Delimiter for u8 {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        let pos = ::memchr::memchr(*self, buf.as_slice());

        Ok(pos.map(move |pos| {
            let mut fir = buf.drain_to(pos + 1);

            let len = fir.len();
            debug_assert!(len > 0);
            fir.get_mut().truncate(len - 1);

            fir.as_slice().to_vec()
        }))
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[*self]);
    }
}

#[test]
fn test_delimiter_u8() {
    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice(&[1, 1, 0, 2, 0, 0]);

    let delimiter = 0;

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![1, 1]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![2]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    let mut v = vec![];
    delimiter.write_delimiter(&mut v);
    assert_eq!(v.as_slice(), &[0u8]);
}

impl Delimiter for char {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        let pos = ::std::str::from_utf8(buf.as_ref())
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?
            .char_indices()
            .find(|&(_, c)| c == *self)
            .map(|(i, _)| i);

        Ok(pos.map(move |pos| {
            let bs = buf.drain_to(pos);
            let _ = buf.drain_to(self.len_utf8());
            bs.as_slice().to_vec()
        }))
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
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
            buf.extend_from_slice(&[code as u8]);
        } else if code < MAX_TWO_B {
            buf.extend_from_slice(&[(code >> 6 & 0x1F) as u8 | TAG_TWO_B,
                                    (code & 0x3F) as u8 | TAG_CONT]);
        } else if code < MAX_THREE_B {
            buf.extend_from_slice(&[(code >> 12 & 0x0F) as u8 | TAG_THREE_B,
                                    (code >> 6 & 0x3F) as u8 | TAG_CONT,
                                    (code & 0x3F) as u8 | TAG_CONT]);
        } else {
            buf.extend_from_slice(&[(code >> 18 & 0x07) as u8 | TAG_FOUR_B,
                                    (code >> 12 & 0x3F) as u8 | TAG_CONT,
                                    (code >> 6 & 0x3F) as u8 | TAG_CONT,
                                    (code & 0x3F) as u8 | TAG_CONT]);
        }
    }
}

#[test]
fn test_delimiter_char() {
    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice("あめ、つち、、".as_bytes());

    let delimiter = '、';

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    let mut v = vec![];
    delimiter.write_delimiter(&mut v);
    assert_eq!(v, "、".as_bytes());
}

/// A line break delimiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineDelimiter {
    /// Carriage return.
    Cr,
    /// Line feed.
    Lf,
    /// Carriage return / line feed.
    CrLf,
}

impl LineDelimiter {
    fn as_slice(&self) -> &'static [u8] {
        static CR: &'static [u8] = &[b'\r'];
        static LF: &'static [u8] = &[b'\n'];
        static CR_LF: &'static [u8] = &[b'\r', b'\n'];

        match *self {
            LineDelimiter::Cr => CR,
            LineDelimiter::Lf => LF,
            LineDelimiter::CrLf => CR_LF,
        }
    }
}

impl Delimiter for LineDelimiter {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        self.as_slice().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_slice());
    }
}

#[test]
fn test_delimiter_line() {
    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice("あめ\r\nつち\r\n\r\n".as_bytes());

    let delimiter = LineDelimiter::CrLf;

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    let mut v = vec![];
    delimiter.write_delimiter(&mut v);
    assert_eq!(v, b"\r\n");
}

impl<'a> Delimiter for &'a [u8] {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        if buf.len() < self.len() {
            return Ok(None);
        }

        for i in 0..buf.len() - self.len() + 1 {
            if buf.as_slice()[i..].starts_with(self) {
                let b = buf.drain_to(i);
                buf.drain_to(self.len());
                return Ok(Some(b.as_ref().to_vec()));
            }
        }

        Ok(None)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Delimiter for Vec<u8> {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        self.as_slice().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_slice());
    }
}

#[test]
fn test_delimiter_vec() {
    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice("あめ#\0#つち#\0##\0#".as_bytes());

    let delimiter = &b"#\0#"[..].to_vec();

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    let mut v = vec![];
    delimiter.write_delimiter(&mut v);
    assert_eq!(v.as_slice(), b"#\0#");
}

impl<'a> Delimiter for &'a str {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl Delimiter for String {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

#[test]
fn test_delimiter_string() {
    let mut buf = EasyBuf::new();
    buf.get_mut().extend("あめ・\nつち・\n・\n".as_bytes());

    let delimiter = "・\n";

    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("あめ".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(),
               Some("つち".as_bytes().to_vec()));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), Some(vec![]));
    assert_eq!(delimiter.pop_buf(&mut buf).unwrap(), None);

    let mut v = vec![];
    delimiter.write_delimiter(&mut v);
    assert_eq!(v, "・\n".as_bytes());
}
