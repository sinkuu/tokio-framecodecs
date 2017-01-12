use tokio_core::io::{Codec, Io, EasyBuf, Framed};
use tokio_proto::pipeline::{ServerProto, ClientProto};
use std::io;

/// Delimitered protocol.
///
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
    type Request = EasyBuf;
    type Response = Vec<u8>;
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
    type Response = EasyBuf;
    type Transport = Framed<T, DelimiterCodec<D>>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(DelimiterCodec(self.0.clone())))
    }
}

/// Protocol codec used by [`DelimiterProto`](./struct.DelimiterProto.html).
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
    type In = EasyBuf;
    type Out = Vec<u8>;

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
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
    /// Removes elements from `buf` including next occurence of this delimiter,
    /// and returns the removed part except the delimiter.
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>>;
    /// Appends this delimiter to `buf`.
    fn write_delimiter(&self, buf: &mut Vec<u8>);
}

impl Delimiter for u8 {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        let pos = ::memchr::memchr(*self, buf.as_slice());

        Ok(pos.map(move |pos| {
            let mut fir = buf.drain_to(pos + 1);

            let len = fir.len();
            debug_assert!(len > 0);
            fir.get_mut().truncate(len - 1);

            fir
        }))
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[*self]);
    }
}

impl Delimiter for char {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        let pos = ::std::str::from_utf8(buf.as_ref())
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?
            .char_indices()
            .find(|&(_, c)| c == *self)
            .map(|(i, _)| i);

        Ok(pos.map(move |pos| {
            let bs = buf.drain_to(pos);
            let _ = buf.drain_to(self.len_utf8());
            bs
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
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        self.as_slice().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_slice());
    }
}

impl<'a> Delimiter for &'a [u8] {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        if buf.len() < self.len() {
            return Ok(None);
        }

        match ::twoway::find_bytes(buf.as_slice(), self) {
            Some(i) => {
                let b = buf.drain_to(i);
                buf.drain_to(self.len());
                Ok(Some(b))
            }

            None => Ok(None),
        }
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Delimiter for Vec<u8> {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        self.as_slice().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_slice());
    }
}

impl<'a> Delimiter for &'a str {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl Delimiter for String {
    fn pop_buf(&self, buf: &mut EasyBuf) -> io::Result<Option<EasyBuf>> {
        self.as_bytes().pop_buf(buf)
    }

    fn write_delimiter(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_delimiter<D, U, V>(d: &D, input: U, mut right: Vec<V>)
        where D: Delimiter,
              U: Into<Vec<u8>>,
              V: Into<Vec<u8>>
    {
        let mut buf = EasyBuf::from(input.into());
        while let Some(f) = d.pop_buf(&mut buf).unwrap() {
            assert_eq!(right.remove(0).into(), f.as_slice().to_vec());
        }
        assert!(buf.len() == 0);
    }

    #[test]
    fn test_delimiter_u8() {
        let d = 0;

        test_delimiter(&d, vec![1, 1, 0, 2, 0, 0], vec![vec![1, 1], vec![2], vec![]]);

        let mut v = vec![];
        d.write_delimiter(&mut v);
        assert_eq!(v.as_slice(), &[0u8]);
    }


    #[test]
    fn test_delimiter_char() {
        let d = '、';

        test_delimiter(&d, "あめ、つち、、", vec!["あめ", "つち", ""]);

        let mut v = vec![];
        d.write_delimiter(&mut v);
        assert_eq!(v, "、".as_bytes());
    }

    #[test]
    fn test_delimiter_line() {
        let d = LineDelimiter::CrLf;

        test_delimiter(&d,
                       "あめ\r\nつち\r\n\r\n",
                       vec!["あめ", "つち", ""]);

        let mut v = vec![];
        d.write_delimiter(&mut v);
        assert_eq!(v, b"\r\n");
    }

    #[test]
    fn test_delimiter_vec() {
        let d = (&b"#\0#"[..]).to_vec();
        test_delimiter(&d,
                       "あめ#\0#つち#\0##\0#",
                       vec!["あめ", "つち", ""]);

        let mut v = vec![];
        d.write_delimiter(&mut v);
        assert_eq!(v.as_slice(), b"#\0#");
    }


    #[test]
    fn test_delimiter_string() {
        let d = "・\n";
        test_delimiter(&d,
                       "あめ・\nつち・\n・\n",
                       vec!["あめ", "つち", ""]);

        let mut v = vec![];
        d.write_delimiter(&mut v);
        assert_eq!(v, "・\n".as_bytes());
    }
}