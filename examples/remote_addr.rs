extern crate framecodecs;
extern crate tokio_proto;
extern crate tokio_core;
extern crate service_fn;

use tokio_core::io::EasyBuf;
use tokio_proto::TcpServer;
use framecodecs::frame::{DelimiterProto, LineDelimiter};
use framecodecs::remote_addr::RemoteAddrProto;
use std::net::SocketAddr;

fn main() {
    let proto = DelimiterProto::new(LineDelimiter::Lf);
    let proto = RemoteAddrProto::new(proto);
    TcpServer::new(proto, "0.0.0.0:8000".parse().unwrap()).serve(|| {
        Ok(service_fn::service_fn(|(addr, line): (SocketAddr, EasyBuf)| {
            println!("{}: {}", addr, String::from_utf8_lossy(line.as_slice()));
            Ok(line.as_slice().to_vec())
        }))
    });
}