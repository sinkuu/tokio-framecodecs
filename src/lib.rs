extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate bytes;
extern crate byteorder;
#[macro_use]
extern crate log;

pub type Frame = tokio_proto::pipeline::Frame<Vec<u8>, (), std::io::Error>;

pub mod fixed_length;
pub use fixed_length::FixedLengthTransport;

pub mod delimiter;
pub use delimiter::DelimiterTransport;

pub mod length_field;
pub use length_field::LengthFieldTransport;

pub mod framed_helper;
