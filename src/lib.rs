// Copyright 2022 - 2023 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
// 
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
// 
// Author: tickbh
// -----
// Created Date: 2023/08/14 04:49:33

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
