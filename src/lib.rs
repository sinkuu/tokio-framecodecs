//! `framecodecs` provides simple protocol implementations to be used with `tokio_proto::pipeline`.

extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate byteorder;
extern crate memchr;
extern crate twoway;

pub mod fixed_length;
pub mod delimiter;
pub mod length_field;
pub mod request_id_field;
pub mod remote_addr;
pub mod varint;