extern crate framecodecs;
extern crate tokio_proto;
extern crate service_fn;

use tokio_proto::TcpServer;
use framecodecs::frame::{DelimiterProto, LineDelimiter};
use framecodecs::remote_addr::RemoteAddrProto;
use std::net::SocketAddr;

fn main() {
    let proto = DelimiterProto::new(LineDelimiter::Lf);
    let proto = RemoteAddrProto::new(proto);
    TcpServer::new(proto, "0.0.0.0:8000".parse().unwrap()).serve(|| {
        Ok(service_fn::service_fn(|(addr, line): (SocketAddr, Vec<u8>)| {
            println!("{}: {}", addr, String::from_utf8_lossy(&line));
            Ok(line)
        }))
    });
}