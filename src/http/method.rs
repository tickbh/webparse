use std::{fmt::Display, io::Write, borrow::Cow};

use crate::{Serialize, WebError, WebResult};

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
}

impl Display for Method {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Options => f.write_str("OPTIONS"),
            Method::Get => f.write_str("GET"),
            Method::Post => f.write_str("POST"),
            Method::Put => f.write_str("PUT"),
            Method::Delete => f.write_str("DELETE"),
            Method::Head => f.write_str("HEAD"),
            Method::Trace => f.write_str("TRACE"),
            Method::Connect => f.write_str("CONNECT"),
            Method::Patch => f.write_str("PATCH"),
            Method::Pri => f.write_str("PRI"),
            Method::None => f.write_str("None"),
            Method::Extension(s) => f.write_str(s.as_str()),
        }
    }
}

impl Serialize for Method {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        match self {
            Method::None => Err(WebError::Serialize("method")),
            _ => Ok(Cow::Owned(format!("{}", self).into_bytes())),
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