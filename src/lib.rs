
#[macro_use] extern crate bitflags;


pub mod binary;
pub mod http;
mod error;
pub mod url;
#[macro_use] mod macros;
mod helper;
mod extensions;
mod serialize;


pub use binary::{Binary, Buf, BinaryMut, BufMut, BinaryRef};

pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Version, Request, Response, HttpError};
pub use http::http2::{self, Http2Error};

pub use error::{WebError, WebResult};
// pub use buffer::Buffer;
pub use url::{Url, Scheme, UrlError};
pub use helper::Helper;
pub use extensions::Extensions;
pub use serialize::Serialize;
