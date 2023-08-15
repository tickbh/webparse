

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

    /// POST
    pub const POST: Method = Method::Post;

    /// PUT
    pub const PUT: Method = Method::Put;

    /// DELETE
    pub const DELETE: Method = Method::Delete;

    /// HEAD
    pub const HEAD: Method = Method::Head;

    /// OPTIONS
    pub const OPTIONS: Method = Method::Options;

    /// CONNECT
    pub const CONNECT: Method = Method::Connect;

    /// PATCH
    pub const PATCH: Method = Method::Patch;

    /// TRACE
    pub const TRACE: Method = Method::Trace;
}