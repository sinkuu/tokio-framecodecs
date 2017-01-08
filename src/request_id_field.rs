//! Converts pipelined protocol to multiplexed protocol by prepending a request id to each frame.

use tokio_core::io::{EasyBuf, Codec, Io, Framed};
use tokio_proto::multiplex::{self, RequestId};
use byteorder::{self, ByteOrder};
use std::marker::PhantomData;
use std::io;

/// `size_of::<RequestId>()`
const SIZE_OF_REQID: usize = 8;

/// A protocol that converts a pipelining codec into a multiplexing codec by prepending a `u64` request id field
/// to every frame of the base codec.
#[derive(Debug, Default, Clone)]
pub struct RequestIdFieldProto<B, C> {
    base: C,
    _byteorder: PhantomData<B>,
}

impl<B, C> RequestIdFieldProto<B, C> where C: Codec + Clone {
    /// Creates a new `RequestIdFieldProto` from a base codec.
    pub fn new(base: C) -> Self {
        RequestIdFieldProto {
            base: base,
            _byteorder: PhantomData,
        }
    }
}

impl<B, C, T> multiplex::ClientProto<T> for RequestIdFieldProto<B, C>
    where C: Codec + Clone + 'static,
          B: byteorder::ByteOrder + 'static,
          T: Io + 'static
{
    type Request = C::Out;
    type Response = C::In;
    type Error = io::Error;
    type Transport = Framed<T, RequestIdFieldCodec<B, C>>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RequestIdFieldCodec::<B, C>::new(self.base.clone())))
    }
}

impl<B, C, T> multiplex::ServerProto<T> for RequestIdFieldProto<B, C>
    where C: Codec + Clone + 'static,
          B: byteorder::ByteOrder + 'static,
          T: Io + 'static
{
    type Request = C::In;
    type Response = C::Out;
    type Error = io::Error;
    type Transport = Framed<T, RequestIdFieldCodec<B, C>>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(RequestIdFieldCodec::<B, C>::new(self.base.clone())))
    }
}

/// Protocol codec used by [`RequestIdFieldProto`](./struct.RequestIdFieldProto.html).
#[derive(Debug, Clone, Default)]
pub struct RequestIdFieldCodec<B, C> {
    base: C,
    reqid: Option<RequestId>,
    _byteorder: PhantomData<B>,
}

impl<B, C> RequestIdFieldCodec<B, C> {
    pub fn new(base: C) -> Self {
        RequestIdFieldCodec {
            base: base,
            reqid: None,
            _byteorder: PhantomData,
        }
    }
}

impl<B, C> Codec for RequestIdFieldCodec<B, C>
    where B: ByteOrder, C: Codec
{
    type In = (RequestId, C::In);
    type Out = (RequestId, C::Out);

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<(RequestId, C::In)>> {
        let reqid = if let Some(id) = self.reqid.take() {
            id
        } else {
            if buf.len() < SIZE_OF_REQID {
                return Ok(None);
            }
            B::read_u64(buf.drain_to(SIZE_OF_REQID).as_slice())
        };

        match self.base.decode(buf) {
            Ok(Some(msg)) => Ok(Some((reqid, msg))),
            Ok(None) => {
                self.reqid = Some(reqid);
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn encode(&mut self, (reqid, msg): (RequestId, C::Out), buf: &mut Vec<u8>) -> io::Result<()> {
        let mut arr = [0u8; SIZE_OF_REQID];
        B::write_u64(&mut arr, reqid);
        buf.extend_from_slice(&arr[..]);
        self.base.encode(msg, buf)
    }
}
