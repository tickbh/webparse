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
// Created Date: 2023/08/15 10:47:56

use std::{fmt::{self}, result, convert::Infallible};

use crate::{http::HttpError, url::UrlError, Http2Error, ws::WsError};

#[derive(Debug)]
pub enum WebError {
    Http(HttpError),
    Http2(Http2Error),
    Ws(WsError),
    Url(UrlError),
    IntoError,
    Extension(&'static str),
    Serialize(&'static str),
    Io(std::io::Error),
}

impl WebError {
    #[inline]
    fn description_str(&self) -> &'static str {
        match self {
            WebError::Url(e) => e.description_str(),
            WebError::Http(e) => e.description_str(),
            WebError::Http2(e) => e.description_str(),
            WebError::Ws(e) => e.description_str(),
            WebError::IntoError => "into value error",
            WebError::Extension(_) => "std error",
            WebError::Serialize(_) => "serialize error",
            WebError::Io(_) => "io error",
            
        }
    }

    pub fn is_partial(&self) -> bool {
        match self {
            WebError::Http(HttpError::Partial) => true,
            _ => false
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

impl From<UrlError> for WebError {
    fn from(e: UrlError) -> Self {
        WebError::Url(e)
    }
}

impl From<Infallible> for WebError {
    fn from(_: Infallible) -> Self {
        WebError::Extension("Infallible")
    }
}

pub type WebResult<T> = result::Result<T, WebError>;
