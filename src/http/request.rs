use url::Url;

use crate::{Buffer, WebResult};
use super::{Method, HeaderMap, Version, skip_empty_lines, parse_method, skip_spaces, parse_version, parse_token, skip_new_line};

pub struct Request {
    parts: Parts,
    body: Buffer
}

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
            parts: Parts { method: Method::NONE, header: HeaderMap::new(), version: Version::None, url: None, path: String::new() }
        }
    }

    pub fn parse(&mut self, buf:&[u8]) -> WebResult<()> {
        let mut buffer = Buffer::new_buf(buf);
        skip_empty_lines(&mut buffer)?;
        self.parts.method = parse_method(&mut buffer)?;
        skip_spaces(&mut buffer)?;
        self.parts.path = parse_token(&mut buffer)?.to_string();
        skip_spaces(&mut buffer)?;
        self.parts.version = parse_version(&mut buffer)?;
        skip_new_line(&mut buffer)?;

        // self.parts.url = 

        Ok(())
    }
}