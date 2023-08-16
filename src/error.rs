use std::{fmt::{self, Result}, result, error::Error};

#[derive(Debug)]
pub enum WebError {
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
    /// Partial
    Partial,

    UrlInvalid,
    UrlCodeInvalid,

    IntoError,
    Extension(),
}

impl WebError {
    #[inline]
    fn description_str(&self) -> &'static str {
        match *self {
            WebError::HeaderName => "invalid header name",
            WebError::HeaderValue => "invalid header value",
            WebError::NewLine => "invalid new line",
            WebError::Status => "invalid response status",
            WebError::Token => "invalid token",
            WebError::Version => "invalid HTTP version",
            WebError::Partial => "invalid HTTP length",
            WebError::UrlInvalid => "invalid Url",
            WebError::UrlCodeInvalid => "invalid Url Code",

            WebError::IntoError => "into value error",
            WebError::Extension() => "std error",
        }
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}

impl From<std::num::ParseIntError> for WebError {
    fn from(_: std::num::ParseIntError) -> Self {
        WebError::Extension()
    }
}

pub type WebResult<T> = result::Result<T, WebError>;
