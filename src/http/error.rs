use std::fmt;



#[derive(Debug)]
pub enum HttpError {
    /// Invalid byte in header name.
    HeaderName,
    /// Invalid byte in header value.
    HeaderValue,
    /// Invalid byte in new line.
    NewLine,
    /// Invalid byte in Response status.
    Status,
    /// Invalid byte where token is required.
    Token,
    /// Invalid byte in HTTP version.
    Version,
    /// 无效的method方法
    Method,
    /// Partial
    Partial,
    /// StatusCode
    InvalidStatusCode,

}

impl HttpError {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
            HttpError::HeaderName => "invalid header name",
            HttpError::HeaderValue => "invalid header value",
            HttpError::NewLine => "invalid new line",
            HttpError::Status => "invalid response status",
            HttpError::Token => "invalid token",
            HttpError::Version => "invalid HTTP version",
            HttpError::Method => "invalid HTTP Method",
            HttpError::Partial => "invalid HTTP length",
            HttpError::InvalidStatusCode => "invalid status code",
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}
