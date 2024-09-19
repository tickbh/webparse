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
// Created Date: 2023/08/15 10:11:50

use std::fmt::Display;

use crate::{WebError, WebResult};
use algorithm::buf::{Bt, BtMut};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Version {
    None,
    Http10,
    Http11,
    Http2,
    Http3,
}

impl Copy for Version {}

impl Version {
    pub const HTTP10: Version = Version::Http10;
    pub const SHTTP10: &'static str = "HTTP/1.0";
    pub const HTTP11: Version = Version::Http11;
    pub const SHTTP11: &'static str = "HTTP/1.1";
    pub const HTTP2: Version = Version::Http2;
    pub const SHTTP2: &'static str = "HTTP/2";
    pub const HTTP3: Version = Version::Http3;
    pub const SHTTP3: &'static str = "HTTP/3";

    pub fn as_str(&self) -> &str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
            Version::Http2 => "HTTP/2",
            Version::Http3 => "HTTP/3",
            Version::None => "None",
        }
    }

    pub fn encode<B: Bt + BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        match self {
            Version::None => Err(WebError::Serialize("version")),
            _ => Ok(buffer.put_slice(&self.as_str().as_bytes())),
        }
    }

    pub fn is_http1(&self) -> bool {
        match self {
            Version::Http10 => true,
            Version::Http11 => true,
            _ => false,
        }
    }

    pub fn is_http2(&self) -> bool {
        match self {
            Version::Http2 => true,
            _ => false,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}
