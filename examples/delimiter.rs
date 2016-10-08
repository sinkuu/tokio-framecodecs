extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate futures;

use framecodecs::DelimiterTransport;
use tokio_proto::client::Client;
use tokio_proto::server::ServerHandle;
use tokio_proto::Message;
use tokio_service::Service;
use tokio_core::reactor::{Core, Handle};
use futures::{Future, Async};
use std::net::SocketAddr;
use std::io;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:32104".parse().unwrap();

    let _server = new_echo_server(&handle, addr);
    let client = new_client(&handle, addr);

    let msg = "from beginning".as_bytes();
    assert_eq!(core.run(client.call(Message::WithoutBody(msg.to_vec()))).unwrap(), msg);
    let msg = "to end".as_bytes();
    assert_eq!(core.run(client.call(Message::WithoutBody(msg.to_vec()))).unwrap(), msg);
}

fn new_client(handle: &Handle, addr: SocketAddr)
    -> Client<Vec<u8>, Vec<u8>, futures::stream::Empty<(), io::Error>, io::Error>
{
    use tokio_core::net::TcpStream;

    let h = handle.clone();

    let new_transport = move || {
        TcpStream::connect(&addr, &h).map(|tcp|
            DelimiterTransport::new(tcp,
                                    b'\0',
                                    Default::default(),
                                    Default::default()))
    };

    tokio_proto::pipeline::connect(new_transport, handle)
}

fn new_echo_server(handle: &Handle, addr: SocketAddr)
    -> io::Result<ServerHandle>
{
    use tokio_proto::pipeline::Server;
    tokio_proto::server::listen(handle, addr, move |stream| {
        let tp = DelimiterTransport::new(stream,
                                         b'\0',
                                         Default::default(),
                                         Default::default());

        Server::new(EchoService, tp)
    })
}

struct EchoService;

impl Service for EchoService {
    type Request = Vec<u8>;
    type Response = Message<Vec<u8>, futures::stream::Empty<(), io::Error>>;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(futures::finished(Message::WithoutBody(req)))
    }

    fn poll_ready(&self) -> Async<()> {
        Async::Ready(())
    }
}
