//! Wrapper protocol for providing remote address of connection.
//!
//! ```rust,no_run
//! extern crate tokio_proto;
//! extern crate framecodecs;
//! extern crate service_fn;
//!
//! use tokio_proto::TcpServer;
//! use tokio_proto::pipeline::Pipeline;
//! use framecodecs::delimiter::{DelimiterCodec, LineDelimiter};
//! use framecodecs::remote_addr::RemoteAddrProto;
//! use std::net::SocketAddr;
//!
//! # fn main() {
//! let codec = DelimiterCodec::new(LineDelimiter::Lf);
//! let proto = RemoteAddrProto::<_, Pipeline>::new(codec);
//! TcpServer::new(proto, "0.0.0.0:8000".parse().unwrap()).serve(|| {
//!     Ok(service_fn::service_fn(|(addr, line): (SocketAddr, Vec<u8>)| {
//!         println!("{}: {}", addr, String::from_utf8_lossy(&line));
//!         Ok(line)
//!     }))
//! });
//! # }
//! ```
use tokio_core::io::{Io, Codec, Framed, EasyBuf};
use tokio_core::net::TcpStream;
use tokio_proto::{pipeline, multiplex};
use tokio_proto::pipeline::Pipeline;
use tokio_proto::multiplex::{Multiplex, RequestId};
use std::io;
use std::net::SocketAddr;
use std::marker::PhantomData;

/// A wrapper protocol provides remote address of connection.
/// This protocol implements only `ServerProto`.
#[derive(Debug, Clone, Copy)]
pub struct RemoteAddrProto<C, Kind> {
    inner: C,
    _kind: PhantomData<Kind>,
}

impl<C: Clone, Kind> RemoteAddrProto<C, Kind> {
    /// Creates a new `RemoteAddrProto` based on codec `inner`.
    #[inline]
    pub fn new(inner: C) -> Self {
        RemoteAddrProto {
            inner: inner,
            _kind: PhantomData,
        }
    }

    fn codec(&self, peer_addr: SocketAddr) -> RemoteAddrCodec<C, Kind> {
        RemoteAddrCodec {
            inner: self.inner.clone(),
            peer_addr: peer_addr,
            _kind: PhantomData,
        }
    }
}

impl<C> pipeline::ServerProto<TcpStream> for RemoteAddrProto<C, Pipeline>
    where C: Codec + Clone + 'static
{
    type Request = (SocketAddr, C::In);
    type Response = C::Out;
    type Error = io::Error;
    type Transport = Framed<TcpStream, RemoteAddrCodec<C, Pipeline>>;
    type BindTransport = io::Result<Self::Transport>;

    #[inline]
    fn bind_transport(&self, io: TcpStream) -> Self::BindTransport {
        let addr = io.peer_addr()?;
        Ok(io.framed(self.codec(addr)))
    }
}

impl<C, In, Out> multiplex::ServerProto<TcpStream> for RemoteAddrProto<C, Multiplex>
    where C: Codec<In = (RequestId, In), Out = (RequestId, Out)> + Clone + 'static,
          In: 'static,
          Out: 'static
{
    type Request = (SocketAddr, In);
    type Response = Out;
    type Error = io::Error;
    type Transport = Framed<TcpStream, RemoteAddrCodec<C, Multiplex>>;
    type BindTransport = io::Result<Self::Transport>;

    #[inline]
    fn bind_transport(&self, io: TcpStream) -> Self::BindTransport {
        let addr = io.peer_addr()?;
        Ok(io.framed(self.codec(addr)))
    }
}

/// Protocol codec used by [`RemoteAddrProto`](./struct.RemoteAddrProto.html).
pub struct RemoteAddrCodec<C, Kind> {
    inner: C,
    peer_addr: SocketAddr,
    _kind: PhantomData<Kind>,
}

impl<C, Kind> RemoteAddrCodec<C, Kind> {
    /// Creates a new `RemoteAddrCodec` based on codec `inner`.
    #[inline]
    pub fn new(inner: C, peer_addr: SocketAddr) -> Self {
        RemoteAddrCodec {
            inner: inner,
            peer_addr: peer_addr,
            _kind: PhantomData,
        }
    }
}

impl<C> Codec for RemoteAddrCodec<C, Pipeline>
    where C: Codec
{
    type In = (SocketAddr, C::In);
    type Out = C::Out;

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        self.inner
            .decode(buf)
            .map(|item| item.map(|item| (self.peer_addr, item)))
    }

    #[inline]
    fn encode(&mut self, item: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        self.inner.encode(item, buf)
    }
}

impl<C, In, Out> Codec for RemoteAddrCodec<C, Multiplex>
    where C: Codec<In = (RequestId, In), Out = (RequestId, Out)>
{
    type In = (RequestId, (SocketAddr, In));
    type Out = (RequestId, Out);

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        self.inner
            .decode(buf)
            .map(|item| item.map(|(reqid, item)| (reqid, (self.peer_addr, item))))
    }

    #[inline]
    fn encode(&mut self, item: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        self.inner.encode(item, buf)
    }
}