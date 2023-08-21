use std::{fmt::{self, Result}, result, error::Error};

use crate::http::HttpError;

#[derive(Debug)]
pub enum WebError {
    Http(HttpError),
    UrlInvalid,
    UrlCodeInvalid,

    IntoError,
    Extension(&'static str),
    Serialize(&'static str),
    Io(std::io::Error),
}

impl WebError {
    #[inline]
    fn description_str(&self) -> &'static str {
        match self {
            // WebError::HeaderName => "invalid header name",
            // WebError::HeaderValue => "invalid header value",
            // WebError::NewLine => "invalid new line",
            // WebError::Status => "invalid response status",
            // WebError::Token => "invalid token",
            // WebError::Version => "invalid HTTP version",
            // WebError::Partial => "invalid HTTP length",
            WebError::UrlInvalid => "invalid Url",
            WebError::UrlCodeInvalid => "invalid Url Code",

            WebError::Http(e) => e.description_str(),
            WebError::IntoError => "into value error",
            WebError::Extension(_) => "std error",
            WebError::Serialize(_) => "serialize error",
            WebError::Io(_) => "io error",
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
        WebError::Extension("parse int error")
    }
}

impl From<std::io::Error> for WebError {
    fn from(e: std::io::Error) -> Self {
        WebError::Io(e)
    }
}

impl From<HttpError> for WebError {
    fn from(e: HttpError) -> Self {
        WebError::Http(e)
    }
}

pub type WebResult<T> = result::Result<T, WebError>;
