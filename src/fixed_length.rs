use tokio_core::io::{Io, FramedIo};
use tokio_proto::{Parse, Serialize, Framed};
use tokio_proto::pipeline;
use bytes::{Buf, MutBuf};
use bytes::buf::BlockBuf;
use futures::{Async, Poll};
use std::io;
use super::Frame;

pub struct FixedLengthTransport<T> {
    inner: Framed<T, Parser, Serializer>,
}

impl<T> FixedLengthTransport<T> where T: Io {
    pub fn new(transport: T, length: usize) -> FixedLengthTransport<T> {
        FixedLengthTransport {
            inner: Framed::new(transport,
                        Parser { length: length },
                        Serializer { length: length },
                        BlockBuf::default(),
                        BlockBuf::default()),
        }
    }
}

impl<T> FramedIo for FixedLengthTransport<T>
    where T: Io
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

pub struct Parser {
    pub length: usize,
}

impl Parse for Parser {
    type Out = Frame;

    fn parse(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
        if buf.len() >= self.length {
            let bs = buf.shift(self.length);
            Some(pipeline::Frame::Message(bs.buf().bytes().into()))
        } else {
            None
        }
    }
}

pub struct Serializer {
    pub length: usize,
}

impl Serialize for Serializer {
    type In = Frame;

    fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
        match frame {
            pipeline::Frame::Message(v) => {
                assert!(v.len() == self.length);
                buf.write_slice(v.as_slice());
            }
            pipeline::Frame::Done => (),
            other => panic!("cannot serialize {:?}", other),
        }
    }
}

#[test]
fn test_fixed_length() {
    let mut p = Parser { length: 5 };

    let mut buf = BlockBuf::default();
    buf.write_slice(b"abcdefghijkl");

    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), b"abcde".to_vec());
    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), b"fghij".to_vec());
    assert!(p.parse(&mut buf).is_none());

    buf.write_slice(b"mno");

    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), b"klmno".to_vec());
}

