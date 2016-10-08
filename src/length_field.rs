use super::Frame;
use tokio_core::io::{Io, FramedIo};
use tokio_proto::{Parse, Serialize, Framed};
use tokio_proto::pipeline;
use bytes::{Buf, MutBuf};
use bytes::buf::BlockBuf;
use futures::{Async, Poll};
use byteorder::ByteOrder;
use std::marker::PhantomData;
use std::io;

pub struct LengthFieldTransport<B, T> {
    pub inner: Framed<T, Parser<B>, Serializer<B>>,
}

impl<B: ByteOrder, T: Io> LengthFieldTransport<B, T> {
    pub fn new(transport: T, field_size: usize) -> LengthFieldTransport<B, T> {
        LengthFieldTransport {
            inner: Framed::new(transport,
                        Parser {
                            field_size: field_size,
                            current_len: None,
                            _byteorder: PhantomData,
                        },
                        Serializer {
                            field_size: field_size,
                            _byteorder: PhantomData,
                        },
                        BlockBuf::default(),
                        BlockBuf::default()),
        }
    }
}

impl<B, T> FramedIo for LengthFieldTransport<B, T>
    where T: Io, B: ByteOrder
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

pub struct Parser<B> {
    pub field_size: usize,
    current_len: Option<usize>,
    _byteorder: PhantomData<B>,
}

impl<B: ByteOrder> Parse for Parser<B> {
    type Out = Frame;

    fn parse(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
        if self.current_len.is_none() && buf.len() >= self.field_size {
            let bs = buf.shift(self.field_size);
            self.current_len = Some(B::read_uint(bs.buf().bytes(), self.field_size) as usize);
        }

        if let Some(cl) = self.current_len {
            if buf.len() >= cl {
                let bs = buf.shift(cl);
                self.current_len = None;
                return Some(pipeline::Frame::Message(bs.buf().bytes().into()));
            }
        }

        None
    }
}

pub struct Serializer<B> {
    pub field_size: usize,
    _byteorder: PhantomData<B>,
}

impl<B: ByteOrder> Serialize for Serializer<B> {
    type In = Frame;

    fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
        match frame {
            pipeline::Frame::Message(v) => {
                buf.write_slice(v.as_slice());

                let mut s = [0; 8];
                B::write_uint(&mut s[..], v.len() as u64, self.field_size);
                buf.write_slice(&s[0 .. self.field_size]);
            }
            pipeline::Frame::Done => (),
            other => panic!("cannot serialize {:?}", other),
        }
    }
}

#[test]
fn test_length_field() {
    use byteorder::BigEndian;
    use std::mem;

    let mut b = [0; 2];
    BigEndian::write_u16(&mut b[..], 3);

    let mut buf = BlockBuf::default();
    buf.write_slice(&b[..]);
    buf.write_slice(b"abc");
    buf.write_slice(&b[..]);
    buf.write_slice(b"def");

    BigEndian::write_u16(&mut b[..], 0);
    buf.write_slice(&b[..]);

    let mut p: Parser<BigEndian> = Parser {
        field_size: mem::size_of::<u16>(),
        current_len: None,
        _byteorder: PhantomData,
    };

    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), &b"abc"[..]);
    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), &b"def"[..]);
    assert_eq!(p.parse(&mut buf).unwrap().unwrap_msg(), &[][..]);
    assert!(p.parse(&mut buf).is_none());
}

