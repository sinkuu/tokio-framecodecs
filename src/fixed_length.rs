use tokio_proto;
use tokio_proto::io::{Parse, Serialize, Framed};
use tokio_proto::proto::pipeline;
use bytes::{Buf, BlockBuf, MutBuf};
use std::io;

pub type FixedLengthTransport<T> = Framed<T, Parser, Serializer>;

pub type Frame = pipeline::Frame<Vec<u8>, io::Error>;

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

pub fn new<T: tokio_proto::io::Stream>(transport: T, length: usize) -> FixedLengthTransport<T> {
    Framed::new(transport,
                Parser { length: length },
                Serializer { length: length },
                BlockBuf::default(),
                BlockBuf::default())
}
