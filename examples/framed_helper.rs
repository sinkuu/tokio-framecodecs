extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate futures;

use framecodecs::fixed_length::FixedLengthTransport;
use framecodecs::framed_helper::{framed_write, framed_read};
use tokio_core::net::TcpStream;
use tokio_proto::pipeline::Frame;
use std::io;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let tcp = core.run(TcpStream::connect(&"127.0.0.1:8000".parse().unwrap(), &handle)).unwrap();

    let ts = FixedLengthTransport::new(tcp, 5);
    let ts = core.run(framed_write(ts, Frame::Message(vec![65, 66, 67, 68, 69]))).unwrap();
    let ts = core.run(framed_write(ts, Frame::Message(vec![70, 71, 72, 73, 74]))).unwrap();
    let (ts, r) = core.run(framed_read(ts)).unwrap();
    println!("{:?}", r);
}

