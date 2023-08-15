use url::Url;

use crate::{Buffer, WebResult};
use super::{Method, HeaderMap, Version};

pub struct Request {
    parts: Parts,
    body: Buffer
}

pub struct Parts {
    pub method: Method,
    pub header: HeaderMap,
    pub version: Version,
    pub url: Option<Url>
}


impl Request {
    
    pub fn new() -> Request {
        Request {
            body: Buffer::new(),
            parts: Parts { method: Method::NONE, header: HeaderMap::new(), version: Version::None, url: None }
        }
    }

    pub fn parse(&mut self, buf:&[u8]) -> WebResult<()> {
        let mut buff = Buffer::new_buf(buf);
        
        Ok(())
    }
}