use tokio_core::io::FramedIo;
use futures::{Future, Poll, Async};
use std::io;

pub fn framed_write<T: FramedIo>(io: T, frame: T::In) -> FramedWrite<T> {
    FramedWrite {
        io: Some(io),
        frame: Some(frame),
    }
}

pub struct FramedWrite<T: FramedIo> {
    io: Option<T>,
    frame: Option<T::In>,
}

impl<T: FramedIo> Future for FramedWrite<T> {
    type Item = T;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<T, io::Error> {
        let mut io = self.io.take().expect("already resolved");

        try!(io.flush());

        if io.poll_write().is_not_ready() {
            self.io = Some(io);
            Ok(Async::NotReady)
        } else {
            try!(io.write(self.frame.take().expect("already resolved")));
            try!(io.flush());
            Ok(Async::Ready(io))
        }
    }
}

pub fn framed_read<T: FramedIo>(io: T) -> FramedRead<T> {
    FramedRead {
        io: Some(io),
    }
}

pub struct FramedRead<T: FramedIo> {
    io: Option<T>,
}

impl<T: FramedIo> Future for FramedRead<T> {
    type Item = (T, T::Out);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(T, T::Out), io::Error> {
        let mut io = self.io.take().expect("already resolved");

        try!(io.flush());

        if io.poll_read().is_not_ready() {
            self.io = Some(io);
            Ok(Async::NotReady)
        } else {
            let r = try!(io.read());
            try!(io.flush());
            Ok(r.map(|res| (io, res)))
        }
    }
}
