

#[derive(Debug, Clone)]
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