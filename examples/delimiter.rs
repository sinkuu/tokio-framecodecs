extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate futures;

use framecodecs::frame::{DelimiterProto, LineDelimiter};

use futures::future::FutureResult;

use tokio_core::io::EasyBuf;
use tokio_core::reactor::Core;
use tokio_proto::{TcpClient, TcpServer};
use tokio_service::Service;

use std::io;
use std::thread;
use std::time::Duration;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:32104".parse().unwrap();

    let proto = DelimiterProto::new(LineDelimiter::Lf);

    thread::spawn(move || {
        TcpServer::new(proto, addr).serve(|| Ok(EchoService));
    });

    thread::sleep(Duration::from_millis(50));

    let client = core.run(TcpClient::new(proto).connect(&addr, &handle)).unwrap();

    let msg = b"Doe, a deer, a female deer";
    assert_eq!(core.run(client.call(msg.to_vec())).unwrap().as_slice().to_vec(), msg);
    let msg = b"Ray, a drop of golden sun";
    assert_eq!(core.run(client.call(msg.to_vec())).unwrap().as_slice().to_vec(), msg);
}

struct EchoService;

impl Service for EchoService {
    type Request = EasyBuf;
    type Response = Vec<u8>;
    type Error = io::Error;
    type Future = FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: EasyBuf) -> Self::Future {
        futures::future::finished(req.as_slice().to_vec())
    }
}
