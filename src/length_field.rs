use tokio_core::io::{Codec, Io, EasyBuf, Framed};
use tokio_proto::pipeline::ServerProto;
use byteorder::ByteOrder;
use std::marker::PhantomData;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LengthFieldProto<B> {
    pub field_size: usize,
    _byteorder: PhantomData<B>,
}

impl<B> LengthFieldProto<B> {
    pub fn new(field_size: usize) -> Self {
        assert!(field_size <= ::std::mem::size_of::<usize>());

        LengthFieldProto {
            field_size: field_size,
            _byteorder: PhantomData,
        }
    }
}

impl<B: ByteOrder + 'static, T: Io + 'static> ServerProto<T> for LengthFieldProto<B> {
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Error = io::Error;
    type Transport = Framed<T, LengthFieldCodec<B>>;
    type BindTransport = io::Result<Self::Transport>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LengthFieldCodec::new(self.field_size)))
    }
}

pub struct LengthFieldCodec<B> {
    field_size: usize,
    current_len: Option<usize>,
    _byteorder: PhantomData<B>,
}

impl<B> LengthFieldCodec<B> {
    pub fn new(field_size: usize) -> LengthFieldCodec<B> {
        assert!(field_size <= ::std::mem::size_of::<usize>());

        LengthFieldCodec {
            field_size: field_size,
            current_len: None,
            _byteorder: PhantomData,
        }
    }
}

impl<B> From<LengthFieldProto<B>> for LengthFieldCodec<B> {
    fn from(proto: LengthFieldProto<B>) -> LengthFieldCodec<B> {
        LengthFieldCodec::new(proto.field_size)
    }
}

impl<B: ByteOrder> Codec for LengthFieldCodec<B> {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    #[inline]
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        if self.current_len.is_none() && buf.len() >= self.field_size {
            let bs = buf.drain_to(self.field_size);
            self.current_len = Some(B::read_uint(bs.as_slice(), self.field_size) as usize);
        }

        if let Some(cl) = self.current_len {
            if buf.len() >= cl {
                let bs = buf.drain_to(cl);
                self.current_len = None;
                return Ok(Some(bs.as_slice().to_vec()));
            }
        }

        Ok(None)
    }

    #[inline]
    fn encode(&mut self, item: Vec<u8>, buf: &mut Vec<u8>) -> io::Result<()> {
        let mut s = [0; 8];
        B::write_uint(&mut s[..], item.len() as u64, self.field_size);
        buf.extend_from_slice(&s[.. self.field_size]);
        buf.extend_from_slice(&item);
        Ok(())
    }
}

#[test]
fn test_length_field() {
    use byteorder::BigEndian;
    use std::mem;

    let mut b = [0; 2];
    BigEndian::write_u16(&mut b[..], 3);

    let mut buf = EasyBuf::new();
    buf.get_mut().extend_from_slice(&b[..]);
    buf.get_mut().extend_from_slice(b"abc");
    buf.get_mut().extend_from_slice(&b[..]);
    buf.get_mut().extend_from_slice(b"def");

    BigEndian::write_u16(&mut b[..], 0);
    buf.get_mut().extend_from_slice(&b[..]);

    let mut p: LengthFieldCodec<BigEndian> = LengthFieldCodec::new(mem::size_of::<u16>());

    assert_eq!(p.decode(&mut buf).unwrap(), Some(b"abc".to_vec()));
    assert_eq!(p.decode(&mut buf).unwrap(), Some(b"def".to_vec()));
    assert_eq!(p.decode(&mut buf).unwrap(), Some(vec![]));
    assert!(p.decode(&mut buf).unwrap().is_none());
}

