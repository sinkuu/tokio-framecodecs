//! Wrapper protocol for providing remote address of connection.
//!
//! ```rust,no_run
//! extern crate tokio_core;
//! extern crate tokio_proto;
//! extern crate framecodecs;
//! extern crate service_fn;
//!
//! use tokio_core::io::EasyBuf;
//! use tokio_proto::TcpServer;
//! use tokio_proto::pipeline::Pipeline;
//! use framecodecs::frame::{DelimiterProto, LineDelimiter};
//! use framecodecs::remote_addr::RemoteAddrProto;
//! use std::net::SocketAddr;
//!
//! fn main() {
//!     let proto = DelimiterProto::new(LineDelimiter::Lf);
//!     let proto = RemoteAddrProto::new(proto);
//!     TcpServer::new(proto, "0.0.0.0:8000".parse().unwrap()).serve(|| {
//!         Ok(service_fn::service_fn(|(addr, line): (SocketAddr, EasyBuf)| {
//!             println!("{}: {}", addr, String::from_utf8_lossy(line.as_slice()));
//!             Ok(line.as_slice().to_vec())
//!         }))
//!     });
//! }
//! ```
use tokio_core::net::TcpStream;
use tokio_proto::{pipeline, multiplex};
use tokio_proto::pipeline::Pipeline;
use tokio_proto::multiplex::{Multiplex, RequestId};
use futures::{Async, Future, Sink, Stream, Poll, StartSend, IntoFuture};
use std::io;
use std::net::SocketAddr;
use std::marker::PhantomData;

/// A wrapper around another protocol that provides remote address of connection.
/// This protocol implements only `ServerProto`.
#[derive(Debug, Clone)]
pub struct RemoteAddrProto<Proto> {
    inner: Proto,
}

impl<Proto> RemoteAddrProto<Proto> {
    /// Creates a new `RemoteAddrProto` based on a protocol `inner`.
    #[inline]
    pub fn new(inner: Proto) -> Self {
        RemoteAddrProto {
            inner: inner,
        }
    }
}

impl<Proto> pipeline::ServerProto<TcpStream> for RemoteAddrProto<Proto>
    where Proto: pipeline::ServerProto<TcpStream> + 'static,
{
    type Request = (SocketAddr, Proto::Request);
    type Response = Proto::Response;
    type Transport = RemoteAddrTransport<Proto::Transport, Pipeline>;
    type BindTransport = NewRemoteAddrTransport<<Proto::BindTransport as IntoFuture>::Future, Pipeline>;

    #[inline]
    fn bind_transport(&self, io: TcpStream) -> Self::BindTransport {
        let peer_addr: io::Result<_> = io.peer_addr();
        NewRemoteAddrTransport::new(self.inner.bind_transport(io).into_future(), peer_addr)
    }
}

impl<Proto> multiplex::ServerProto<TcpStream> for RemoteAddrProto<Proto>
    where Proto: multiplex::ServerProto<TcpStream> + 'static,
{
    type Request = (SocketAddr, Proto::Request);
    type Response = Proto::Response;
    type Transport = RemoteAddrTransport<Proto::Transport, Multiplex>;
    type BindTransport = NewRemoteAddrTransport<<Proto::BindTransport as IntoFuture>::Future, Multiplex>;

    #[inline]
    fn bind_transport(&self, io: TcpStream) -> Self::BindTransport {
        let peer_addr: io::Result<_> = io.peer_addr();
        NewRemoteAddrTransport::new(self.inner.bind_transport(io).into_future(), peer_addr)
    }
}

#[derive(Debug)]
pub struct NewRemoteAddrTransport<Transport, Kind> {
    payload: Option<(Transport, io::Result<SocketAddr>)>,
    _kind: PhantomData<Kind>,
}

impl<Transport, Kind> NewRemoteAddrTransport<Transport, Kind> {
    fn new(transport: Transport, peer_addr: io::Result<SocketAddr>) -> Self {
        NewRemoteAddrTransport {
            payload: Some((transport, peer_addr)),
            _kind: PhantomData,
        }
    }
}

impl<Transport, Kind> Future for NewRemoteAddrTransport<Transport, Kind>
    where Transport: Future<Error = io::Error>
{
    type Item = RemoteAddrTransport<Transport::Item, Kind>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.payload.take() {
            Some((mut transport, peer_addr)) => {
                let t = match transport.poll() {
                    Ok(Async::NotReady) => {
                        self.payload = Some((transport, peer_addr));
                        return Ok(Async::NotReady);
                    }
                    Ok(Async::Ready(t)) => t,
                    Err(e) => return Err(e),
                };
                Ok(Async::Ready(RemoteAddrTransport::new(t, peer_addr?)))
            }
            None => {
                panic!("cannot be polled twice");
            }
        }
    }
}

/// The transport used by [`RemoteAddrProto`](./struct.RemoteAddrProto.html).
pub struct RemoteAddrTransport<Transport, Kind> {
    inner: Transport,
    peer_addr: SocketAddr,
    _kind: PhantomData<Kind>,
}

impl<Transport, Kind> RemoteAddrTransport<Transport, Kind> {
    /// Creates a new `RemoteAddrTransport` based on a transport `inner`.
    #[inline]
    pub fn new(inner: Transport, peer_addr: SocketAddr) -> Self {
        RemoteAddrTransport {
            inner: inner,
            peer_addr: peer_addr,
            _kind: PhantomData,
        }
    }
}

impl<Transport> Stream for RemoteAddrTransport<Transport, Pipeline>
    where Transport: Stream
{
    type Item = (SocketAddr, Transport::Item);
    type Error = Transport::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll().map(|async| async.map(|ok| ok.map(|item| (self.peer_addr, item))))
    }
}

impl<Transport> Sink for RemoteAddrTransport<Transport, Pipeline>
    where Transport: Sink,
{
    type SinkItem = Transport::SinkItem;
    type SinkError = Transport::SinkError;

    #[inline]
    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.inner.start_send(item)
    }

    #[inline]
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.poll_complete()
    }
}

impl<Transport, T> Stream for RemoteAddrTransport<Transport, Multiplex>
    where Transport: Stream<Item = (RequestId, T)>
{
    type Item = (RequestId, (SocketAddr, T));
    type Error = Transport::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.inner.poll().map(|async| async.map(|ok| ok.map(|(id, item)| (id, (self.peer_addr, item)))))
    }
}

impl<Transport> Sink for RemoteAddrTransport<Transport, Multiplex>
    where Transport: Sink,
{
    type SinkItem = Transport::SinkItem;
    type SinkError = Transport::SinkError;

    #[inline]
    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.inner.start_send(item)
    }

    #[inline]
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.poll_complete()
    }
}
