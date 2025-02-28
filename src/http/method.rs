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
// Created Date: 2023/08/15 10:03:23

use std::{fmt::Display, str::FromStr};

use crate::{WebError, WebResult};
use algorithm::buf::{Bt, BtMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    None,
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
    Extension(String),
}

impl Method {
    pub const NONE: Method = Method::None;
    /// GET
    pub const GET: Method = Method::Get;
    pub const SGET: &'static str = "GET";

    /// POST
    pub const POST: Method = Method::Post;
    pub const SPOST: &'static str = "POST";

    /// PUT
    pub const PUT: Method = Method::Put;
    pub const SPUT: &'static str = "PUT";

    /// DELETE
    pub const DELETE: Method = Method::Delete;
    pub const SDELETE: &'static str = "DELETE";

    /// HEAD
    pub const HEAD: Method = Method::Head;
    pub const SHEAD: &'static str = "HEAD";

    /// OPTIONS
    pub const OPTIONS: Method = Method::Options;
    pub const SOPTIONS: &'static str = "OPTIONS";

    /// CONNECT
    pub const CONNECT: Method = Method::Connect;
    pub const SCONNECT: &'static str = "CONNECT";

    /// PATCH
    pub const PATCH: Method = Method::Patch;
    pub const SPATCH: &'static str = "PATCH";

    /// TRACE
    pub const TRACE: Method = Method::Trace;
    pub const STRACE: &'static str = "TRACE";
}

impl Method {
    pub fn is_nobody(&self) -> bool {
        match self {
            Method::Get => true,
            Method::Head => true,
            Method::Options => true,
            Method::Connect => true,
            _ => false,
        }
    }

    pub fn res_nobody(&self) -> bool {
        match self {
            Method::Head => true,
            Method::Options => true,
            Method::Connect => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Method::Options => "OPTIONS",
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Head => "HEAD",
            Method::Trace => "TRACE",
            Method::Connect => "CONNECT",
            Method::Patch => "PATCH",
            Method::None => "None",
            Method::Extension(s) => &s.as_str(),
        }
    }

    pub fn encode<B: Bt + BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        match self {
            Method::None => Err(WebError::Serialize("method")),
            _ => Ok(buffer.put_slice(self.as_str().as_bytes())),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl TryFrom<&str> for Method {
    type Error = WebError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            Method::SGET => Ok(Method::GET),
            Method::SPOST => Ok(Method::POST),
            Method::SPUT => Ok(Method::PUT),
            Method::SDELETE => Ok(Method::DELETE),
            Method::SHEAD => Ok(Method::HEAD),
            Method::SOPTIONS => Ok(Method::OPTIONS),
            Method::SCONNECT => Ok(Method::CONNECT),
            Method::SPATCH => Ok(Method::PATCH),
            Method::STRACE => Ok(Method::TRACE),
            _ => Err(WebError::Http(crate::HttpError::Method)),
        }
    }
}

impl FromStr for Method {
    type Err = WebError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Method::try_from(&*s.to_uppercase())
    }
}
