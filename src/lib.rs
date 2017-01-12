//! `framecodecs` provides simple protocol implementations to be used with tokio-proto.

#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate byteorder;
extern crate memchr;
extern crate twoway;

pub mod frame;
pub mod request_id_field;
pub mod remote_addr;
pub mod decode_to_vec;