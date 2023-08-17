

#[derive(Debug, Eq, PartialEq)]
pub enum Version {
    None,
    Http10,
    Http11,
    Http2,
    Http3,
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