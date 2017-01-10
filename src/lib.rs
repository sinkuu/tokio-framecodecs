//! `framecodecs` provides simple, non-streaming protocol implementations to be used with `tokio_proto`.

extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate byteorder;
extern crate memchr;
#[macro_use]
extern crate log;

pub mod fixed_length;
pub mod delimiter;
pub mod length_field;
pub mod request_id_field;
pub mod remote_addr;
