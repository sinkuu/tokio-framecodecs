extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate futures;

use framecodecs::delimiter::{DelimiterProto, LineDelimiter};

use futures::future::FutureResult;

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

    thread::sleep(Duration::from_millis(100));

    let mut client = core.run(TcpClient::new(proto).connect(&addr, &handle)).unwrap();

    let msg = b"Doe, a deer, a female deer";
    assert_eq!(core.run(client.call(msg.to_vec())).unwrap(), msg);
    let msg = b"Ray, a drop of golden sun";
    assert_eq!(core.run(client.call(msg.to_vec())).unwrap(), msg);
}

struct EchoService;

impl Service for EchoService {
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Error = io::Error;
    type Future = FutureResult<Self::Response, Self::Error>;

    fn call(&mut self, req: Vec<u8>) -> Self::Future {
        futures::future::finished(req)
    }
}
