use std::{fmt::Display, io::Write, borrow::Cow};

use crate::{Serialize, WebError, WebResult};



#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Version {
    None,
    Http10,
    Http11,
    Http2,
    Http3,
}

impl Copy for Version {

}

impl Version {
    pub const  HTTP10: Version = Version::Http10;
    pub const SHTTP10: &'static str = "HTTP/1.0";
    pub const  HTTP11: Version = Version::Http11;
    pub const SHTTP11: &'static str = "HTTP/1.1";
    pub const  HTTP2: Version = Version::Http2;
    pub const SHTTP2: &'static str = "HTTP/2";
    pub const  HTTP3: Version = Version::Http3;
    pub const SHTTP3: &'static str = "HTTP/3";
}

impl Display for Version {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Http10 => f.write_str("HTTP/1.0"),
            Version::Http11 => f.write_str("HTTP/1.1"),
            Version::Http2 => f.write_str("HTTP/2"),
            Version::Http3 => f.write_str("HTTP/3"),
            Version::None => f.write_str("None"),
        }
    }
}

impl Serialize for Version {
    fn serialize(&self, buffer: &mut crate::Buffer) -> crate::WebResult<()> {
        match self {
            Version::None => return Err(WebError::Serialize("version")),
            _ => buffer.write(format!("{}", self).as_bytes()).map_err(WebError::from)?,
        };
        Ok(())
    }


    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        match self {
            Version::None => Err(WebError::Serialize("version")),
            _ => Ok(Cow::Owned(format!("{}", self).into_bytes()))
        }
    }
}