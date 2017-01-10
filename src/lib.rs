//! `framecodecs` provides simple protocol implementations to be used with `tokio_proto::pipeline`.

extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate byteorder;
extern crate memchr;
#[macro_use]
extern crate log;

pub mod fixed_length;
pub use fixed_length::FixedLengthProto;

pub mod delimiter;
pub use delimiter::DelimiterProto;

pub mod length_field;
pub use length_field::LengthFieldProto;

pub mod request_id_field;

pub mod remote_addr;