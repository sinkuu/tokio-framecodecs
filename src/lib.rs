extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate bytes;
extern crate byteorder;
#[macro_use]
extern crate log;

pub type Frame = tokio_proto::pipeline::Frame<Vec<u8>, (), std::io::Error>;

pub mod fixed_length;
pub mod delimiter;
pub mod length_field;

pub mod framed_helper;
