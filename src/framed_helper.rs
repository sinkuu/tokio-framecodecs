use tokio_core::io::FramedIo;
use futures::{Future, Poll, Async};
use std::io;
use std::mem;

pub fn framed_write<T: FramedIo>(io: T, frame: T::In) -> FramedWrite<T> {
    FramedWrite::Writing {
        io: io,
        frame: Some(frame),
    }
}

pub enum FramedWrite<T: FramedIo> {
    Writing {
        io: T,
        frame: Option<T::In>,
    },
    Empty,
}

impl<T: FramedIo> Future for FramedWrite<T> {
    type Item = T;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<T, io::Error> {
        match *self {
            FramedWrite::Writing { ref mut io, ref mut frame } => {
                try!(io.flush());
                if io.poll_write().is_not_ready() {
                    return Ok(Async::NotReady);
                } else {
                    if let Some(frame) = frame.take() {
                        try!(io.write(frame));
                        try!(io.flush());
                    } else {
                        unreachable!()
                    }
                }
            }

            FramedWrite::Empty => panic!("poll FramedWrite after it's done"),
        }

        match mem::replace(self, FramedWrite::Empty) {
            FramedWrite::Writing { io, ..} => Ok(Async::Ready(io)),
            FramedWrite::Empty => unreachable!(),
        }
    }
}

pub fn framed_read<T: FramedIo>(io: T) -> FramedRead<T> {
    FramedRead::Reading {
        io: io,
        res: None,
    }
}

pub enum FramedRead<T: FramedIo> {
    Reading {
        io: T,
        res: Option<T::Out>,
    },
    Empty,
}

impl<T: FramedIo> Future for FramedRead<T> {
    type Item = (T, T::Out);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(T, T::Out), io::Error> {
        match *self {
            FramedRead::Reading { ref mut io, ref mut res } => {
                try!(io.flush());

                if io.poll_read().is_not_ready() {
                    return Ok(Async::NotReady);
                } else {
                    if let Async::Ready(r) = try!(io.read()) {
                        *res = Some(r);
                    } else {
                        return Ok(Async::NotReady);
                    }
                }

                try!(io.flush());
            }

            FramedRead::Empty => panic!("poll FramedRead after it's done"),
        }

        match mem::replace(self, FramedRead::Empty) {
            FramedRead::Reading { io, res: Some(r) } => Ok(Async::Ready((io, r))),
            _ => unreachable!(),
        }
    }
}
