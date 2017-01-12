use tokio_core::io::{EasyBuf, Io};
use tokio_proto::{pipeline, multiplex};
use tokio_proto::pipeline::Pipeline;
use tokio_proto::multiplex::{RequestId, Multiplex};
use futures::{Future, IntoFuture, Stream, Sink, StartSend, Poll, Async};
use std::marker::PhantomData;
use std::io;

pub struct DecodeToVecProto<P> {
    inner: P,
}

impl<P> DecodeToVecProto<P> {
    pub fn new(inner: P) -> Self {
        DecodeToVecProto { inner: inner }
    }
}

impl<P, T> pipeline::ServerProto<T> for DecodeToVecProto<P>
    where P: pipeline::ServerProto<T, Request = EasyBuf>,
          T: Io + 'static
{
    type Request = Vec<u8>;
    type Response = P::Response;
    type Transport = DecodeToVecTransport<P::Transport, Pipeline>;
    type BindTransport = DecodeToVecBind<<P::BindTransport as IntoFuture>::Future, Pipeline>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        DecodeToVecBind::new(self.inner.bind_transport(io).into_future())
    }
}

impl<P, T> multiplex::ServerProto<T> for DecodeToVecProto<P>
    where P: multiplex::ServerProto<T, Request = EasyBuf>,
          T: Io + 'static
{
    type Request = Vec<u8>;
    type Response = P::Response;
    type Transport = DecodeToVecTransport<P::Transport, Multiplex>;
    type BindTransport = DecodeToVecBind<<P::BindTransport as IntoFuture>::Future, Multiplex>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        DecodeToVecBind::new(self.inner.bind_transport(io).into_future())
    }
}

pub struct DecodeToVecBind<F, Kind> {
    fut: F,
    _kind: PhantomData<Kind>,
}

impl<F, Kind> DecodeToVecBind<F, Kind> {
    pub fn new(fut: F) -> Self {
        DecodeToVecBind {
            fut: fut,
            _kind: PhantomData,
        }
    }
}

impl<F, Kind> Future for DecodeToVecBind<F, Kind>
    where F: Future
{
    type Item = DecodeToVecTransport<F::Item, Kind>;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let inner = try_ready!(self.fut.poll());
        Ok(Async::Ready(DecodeToVecTransport::new(inner)))
    }
}

pub struct DecodeToVecTransport<T, Kind> {
    inner: T,
    _kind: PhantomData<Kind>,
}

impl<T, Kind> DecodeToVecTransport<T, Kind> {
    pub fn new(inner: T) -> Self {
        DecodeToVecTransport {
            inner: inner,
            _kind: PhantomData,
        }
    }
}

impl<T> Stream for DecodeToVecTransport<T, Pipeline>
    where T: Stream<Item = EasyBuf, Error = io::Error>
{
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(try_ready!(self.inner.poll()).map(|buf| buf.as_slice().to_vec())))
    }
}

impl<T> Sink for DecodeToVecTransport<T, Pipeline>
    where T: Sink
{
    type SinkItem = T::SinkItem;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.poll_complete()
    }
}

impl<T> Stream for DecodeToVecTransport<T, Multiplex>
    where T: Stream<Item = (RequestId, EasyBuf), Error = io::Error>
{
    type Item = (RequestId, Vec<u8>);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(try_ready!(self.inner.poll())
            .map(|(id, buf)| (id, buf.as_slice().to_vec()))))
    }
}

impl<T> Sink for DecodeToVecTransport<T, Multiplex>
    where T: Sink
{
    type SinkItem = T::SinkItem;
    type SinkError = T::SinkError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.poll_complete()
    }
}