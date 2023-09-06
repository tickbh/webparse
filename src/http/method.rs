use std::{fmt::Display, io::Write, borrow::Cow};

use crate::{Serialize, WebError, WebResult, Buf, BufMut, MarkBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    //http2 协议头
    Pri,
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

    /// PRI
    pub const PRI: Method = Method::Pri;
    pub const SPRI: &'static str = "PRI";
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
            Method::Pri => "PRI",
            Method::None => "None",
            Method::Extension(s) => &s.as_str(),
        }
    }
}

impl Display for Method {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl Serialize for Method {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        match self {
            Method::None => Err(WebError::Serialize("method")),
            _ => Ok(buffer.put_slice(self.as_str().as_bytes())),
        }
    }
}

impl TryFrom<&str> for Method {
    type Error=WebError;

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
            _ => {
                Ok(Method::Extension(value.to_string()))
            }
        }
    }
}