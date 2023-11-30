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
// Created Date: 2023/08/21 06:03:19

use std::fmt;



#[derive(Debug)]
pub enum HttpError {
    /// 数据太小不足以支持读
    BufTooShort,
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
    /// Scheme 太长了
    SchemeTooLong,

}

impl HttpError {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
            HttpError::BufTooShort => "buf too short",
            HttpError::HeaderName => "invalid header name",
            HttpError::HeaderValue => "invalid header value",
            HttpError::NewLine => "invalid new line",
            HttpError::Status => "invalid response status",
            HttpError::Token => "invalid token",
            HttpError::Version => "invalid HTTP version",
            HttpError::Method => "invalid HTTP Method",
            HttpError::Partial => "invalid HTTP length",
            HttpError::InvalidStatusCode => "invalid status code",
            HttpError::SchemeTooLong => "scheme too long",
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}
