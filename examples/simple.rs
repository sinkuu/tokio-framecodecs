extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate futures;

use framecodecs::fixed_length::FixedLengthCodec;
use tokio_core::net::TcpStream;
use tokio_core::io::Io;
use futures::{Stream, Sink};

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let tcp = core.run(TcpStream::connect(&"127.0.0.1:8000".parse().unwrap(), &handle)).unwrap();

    let ts = tcp.framed(FixedLengthCodec::new(5));
    let ts = core.run(ts.send(vec![65, 66, 67, 68, 69])).unwrap();
    let ts = core.run(ts.send(vec![70, 71, 72, 73, 74])).unwrap();

    core.run(ts.for_each(|v| {
            println!("{:?}", v);
            Ok(())
        }))
        .unwrap();
}
