
#[macro_use] extern crate bitflags;


pub mod binary;
pub mod http;
mod error;
pub mod url;
#[macro_use] mod macros;
mod helper;
mod extensions;
mod serialize;


pub use binary::{Binary, Buf, MarkBuf, BinaryMut, BufMut};

pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Version, Request, Response, HttpError};
pub use http::http2::{Http2Error};

pub use error::{WebError, WebResult};
// pub use buffer::Buffer;
pub use url::{Url, Scheme, UrlError};
pub use helper::Helper;
pub use extensions::Extensions;
pub use serialize::Serialize;
