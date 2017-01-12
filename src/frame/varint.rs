use tokio_core::io::{Codec, Io, EasyBuf, Framed};
use tokio_proto::pipeline;
use std::io;
use std::mem;

/// Length field based protocol using Protobuf's base 128 varint.
///
/// A protocol such that every frame has length field prepended in
/// [Protobuf's base 128 varint](https://developers.google.com/protocol-buffers/docs/encoding#varints)
/// format.
#[derive(Debug, Clone, Default)]
pub struct VarIntLengthFieldProto;

impl VarIntLengthFieldProto {
    pub fn new() -> VarIntLengthFieldProto {
        VarIntLengthFieldProto
    }

    fn codec(&self) -> VarIntLengthFieldCodec {
        VarIntLengthFieldCodec::default()
    }
}

impl<T> pipeline::ClientProto<T> for VarIntLengthFieldProto
    where T: Io + 'static
{
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Transport = Framed<T, VarIntLengthFieldCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(self.codec()))
    }
}

impl<T> pipeline::ServerProto<T> for VarIntLengthFieldProto
    where T: Io + 'static
{
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type Transport = Framed<T, VarIntLengthFieldCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(self.codec()))
    }
}

/// Protocol codec used by [`VarIntLengthFieldProto`](./struct.VarIntLengthFieldProto.html).
#[derive(Debug, Clone, Default)]
pub struct VarIntLengthFieldCodec {
    pos: usize,
    size: Option<usize>,
}

impl VarIntLengthFieldCodec {
    pub fn new() -> VarIntLengthFieldCodec {
        Default::default()
    }
}

impl Codec for VarIntLengthFieldCodec {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Vec<u8>>> {
        if let Some(size) = self.size {
            if buf.len() >= size {
                self.size = None;
                let b = buf.drain_to(size);
                Ok(Some(b.as_slice().to_vec()))
            } else {
                Ok(None)
            }
        } else {
            for pos in self.pos..buf.len() {
                if &buf.as_slice()[pos] & 0x80 != 0 {
                    self.pos += 1;
                } else {
                    let varint = buf.drain_to(self.pos + 1);
                    let mut size: usize = 0;
                    for (i, e) in varint.as_slice().iter().enumerate() {
                        size += ((e & 0x7F) as usize) << (i * 7);
                    }
                    self.size = Some(size);
                    self.pos = 0;
                    return self.decode(buf);
                }
            }
            Ok(None)
        }
    }

    fn encode(&mut self, item: Vec<u8>, buf: &mut Vec<u8>) -> io::Result<()> {
        let mut size = item.len();
        buf.reserve_exact(size +
                          (mem::size_of::<usize>() * 8 - (size.leading_zeros() as usize) + 6) / 7);
        while size > 0 {
            buf.push(((size & 0x7F) as u8) | if size >= 0x80 { 0x80 } else { 0 });
            size = size >> 7;
        }
        buf.extend(item);
        Ok(())
    }
}

#[test]
fn test_varintlengthfield() {
    let mut p = VarIntLengthFieldCodec::new();

    let mut buf = EasyBuf::new();
    buf.get_mut().extend(&[1, 65, 2, 65, 66]);

    assert_eq!(p.decode(&mut buf).unwrap().unwrap(), vec![65]);
    assert_eq!(p.decode(&mut buf).unwrap().unwrap(), vec![65, 66]);
    assert!(buf.as_slice().is_empty());
    assert!(p.decode(&mut buf).unwrap().is_none());

    p.encode(vec![0, 1, 2], &mut buf.get_mut()).unwrap();
    assert_eq!(p.decode(&mut buf).unwrap().unwrap(), vec![0, 1, 2]);
    assert!(p.decode(&mut buf).unwrap().is_none());

    let mut test_large = |len| {
        let data: Vec<_> = (0..10).cycle().take(len).collect();
        p.encode(data.clone(), &mut buf.get_mut()).unwrap();
        assert_eq!(p.decode(&mut buf).unwrap().unwrap(), data);
    };

    test_large(128);
    test_large(300);
    test_large(1000);
}