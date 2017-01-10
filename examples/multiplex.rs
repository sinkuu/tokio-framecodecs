extern crate framecodecs;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate service_fn;
extern crate byteorder;

use tokio_core::reactor::Core;
use tokio_proto::{TcpServer, TcpClient};
use tokio_service::Service;
use framecodecs::request_id_field::RequestIdFieldProto;
use framecodecs::length_field::LengthFieldCodec;
use std::thread;
use std::time::Duration;

fn main() {
    let proto =
        RequestIdFieldProto::<byteorder::BigEndian, _>::new(LengthFieldCodec::<byteorder::BigEndian>::new(4));
    {
        let proto = proto.clone();

        thread::spawn(move || {
            TcpServer::new(proto, "0.0.0.0:8000".parse().unwrap()).serve(|| {
                Ok(service_fn::service_fn(|x: Vec<u8>| Ok(x.into_iter().rev().collect())))
            })
        });
    }

    thread::sleep(Duration::from_millis(50));

    let mut reactor = Core::new().unwrap();
    let handle = reactor.handle();

    let client =
        reactor.run(TcpClient::new(proto).connect(&"0.0.0.0:8000".parse().unwrap(), &handle))
            .unwrap();

    assert_eq!(reactor.run(client.call(b"hello".to_vec())).unwrap(),
               b"olleh".to_vec());
    assert_eq!(reactor.run(client.call(b"good bye".to_vec())).unwrap(),
               b"eyb doog".to_vec());
}
