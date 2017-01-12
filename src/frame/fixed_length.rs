use tokio_core::io::{Codec, Io, EasyBuf, Framed};
use tokio_proto::pipeline::{ServerProto, ClientProto};
use std::io;

/// Fixed-length protocol.
///
/// A protocol such that frames are continuous and have the same specified length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedLengthProto {
    pub length: usize,
}

impl FixedLengthProto {
    pub fn new(length: usize) -> FixedLengthProto {
        FixedLengthProto { length: length }
    }

    fn codec(&self) -> FixedLengthCodec {
        FixedLengthCodec { length: self.length }
    }
}

impl<T: Io + 'static> ServerProto<T> for FixedLengthProto {
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Transport = Framed<T, FixedLengthCodec>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(self.codec()))
    }
}

impl<T: Io + 'static> ClientProto<T> for FixedLengthProto {
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Transport = Framed<T, FixedLengthCodec>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(self.codec()))
    }
}

/// Protocol codec used by [`FixedLengthProto`](./struct.FixedLengthProto.html).
#[derive(Debug, Clone)]
pub struct FixedLengthCodec {
    length: usize,
}

impl FixedLengthCodec {
    pub fn new(length: usize) -> FixedLengthCodec {
        FixedLengthCodec { length: length }
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl Codec for FixedLengthCodec {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        Ok(if buf.len() >= self.length {
            let bs = buf.drain_to(self.length);
            Some(bs.as_slice().to_vec())
        } else {
            None
        })
    }

    #[inline]
    fn encode(&mut self, item: Vec<u8>, buf: &mut Vec<u8>) -> io::Result<()> {
        assert_eq!(item.len(), self.length);
        buf.extend_from_slice(&item);
        Ok(())
    }
}

#[test]
fn test_fixed_length() {
    let mut p = FixedLengthCodec { length: 5 };

    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice(b"abcdefghijkl");

    assert_eq!(p.decode(&mut buf).unwrap(), Some(b"abcde".to_vec()));
    assert_eq!(p.decode(&mut buf).unwrap(), Some(b"fghij".to_vec()));
    assert!(p.decode(&mut buf).unwrap().is_none());

    buf.get_mut().extend_from_slice(b"mno");

    assert_eq!(p.decode(&mut buf).unwrap(), Some(b"klmno".to_vec()));
}
