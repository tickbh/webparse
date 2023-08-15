use url::Url;

use crate::{Buffer, WebResult};
use super::{Method, HeaderMap, Version, Helper};

#[derive(Debug)]
pub struct Request {
    parts: Parts,
    body: Buffer,
    partial: bool,
}

#[derive(Debug)]
pub struct Parts {
    pub method: Method,
    pub header: HeaderMap,
    pub version: Version,
    pub url: Option<Url>,
    pub path: String,
}


impl Request {
    
    pub fn new() -> Request {
        Request {
            body: Buffer::new(),
            partial: false,
            parts: Parts { method: Method::NONE, header: HeaderMap::new(), version: Version::None, url: None, path: String::new() }
        }
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    pub fn parse(&mut self, buf:&[u8]) -> WebResult<()> {
        self.partial = true;
        let mut buffer = Buffer::new_buf(buf);
        Helper::skip_empty_lines(&mut buffer)?;
        self.parts.method = Helper::parse_method(&mut buffer)?;
        Helper::skip_spaces(&mut buffer)?;
        self.parts.path = Helper::parse_token(&mut buffer)?.to_string();
        Helper::skip_spaces(&mut buffer)?;
        self.parts.version = Helper::parse_version(&mut buffer)?;
        Helper::skip_new_line(&mut buffer)?;
        Helper::parse_header(&mut buffer, &mut self.parts.header)?;
        self.partial = false;
        Ok(())
    }
}