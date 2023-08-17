
use crate::{Buffer, WebResult, Url, Helper, WebError};
use super::{Method, HeaderMap, Version};

#[derive(Debug)]
pub struct Request {
    parts: Parts,
    body: Vec<u8>,
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
            body: Vec::new(),
            partial: false,
            parts: Parts { method: Method::NONE, header: HeaderMap::new(), version: Version::None, url: None, path: String::new() }
        }
    }
    
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    pub fn get_host(&self) -> Option<String> {
        self.parts.get_host()
    }

    pub fn get_connect_url(&self) -> Option<String> {
        self.parts.get_connect_url()
    }

    pub fn get_body_len(&self) -> usize {
        self.parts.header.get_body_len()
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    fn parse_connect_by_host(url: &mut Url, h: &String) -> WebResult<()> {
        // Host中存在端口号, 则直接取端口号
        let vec: Vec<&str> = h.split(":").collect();
        if vec.len() == 1 {
            url.domain = Some(vec[0].to_string());
            url.port = Some(80);
        } else if vec.len() == 2 {
            url.domain = Some(vec[0].to_string());
            url.port = Some(vec[1].parse().map_err(WebError::from)?);
        } else {
            return Err(WebError::IntoError);
        }

        Ok(())
    }

    pub fn parse_buffer(&mut self, buffer:&mut Buffer) -> WebResult<()> {
        Helper::skip_empty_lines(buffer)?;
        self.parts.method = Helper::parse_method(buffer)?;
        Helper::skip_spaces(buffer)?;
        self.parts.path = Helper::parse_token(buffer)?.to_string();
        Helper::skip_spaces(buffer)?;
        self.parts.version = Helper::parse_version(buffer)?;
        Helper::skip_new_line(buffer)?;
        Helper::parse_header(buffer, &mut self.parts.header)?;
        self.partial = false;
        self.parts.url = Some(match self.parts.method {
            // Connect 协议, Path则为连接地址, 
            Method::Connect => {
                let mut url = Url::new();
                Self::parse_connect_by_host(&mut url, &self.parts.path)?;
                url
            }
            _ => {
                let mut url = Url::parse(&self.parts.path)?;
                if url.domain.is_none() {
                    match self.parts.header.get_host() {
                        Some(h) => {
                            Self::parse_connect_by_host(&mut url, &h)?;
                        }
                        _ => (),
                    }
                }
                url
            }
        });
        Ok(())
    }

    pub fn parse(&mut self, buf:&[u8]) -> WebResult<()> {
        self.partial = true;
        let mut buffer = Buffer::new_buf(buf);
        self.parse_buffer(&mut buffer)
    }
}

impl Parts {
    pub fn get_host(&self) -> Option<String> {
        if self.url.is_some() {
            let url = self.url.as_ref().unwrap();
            if url.domain.is_some() {
                return url.domain.clone();
            }
        }
        self.header.get_host()
    }

    // like wwww.baidu.com:80, wwww.google.com:443
    pub fn get_connect_url(&self) -> Option<String> {
        if self.url.is_none() {
            return None;
        }
        let url = self.url.as_ref().unwrap();
        if url.domain.is_some() && url.port.is_some() {
            Some(format!("{}:{}", url.domain.as_ref().unwrap(), url.port.as_ref().unwrap()))
        } else {
            None
        }
    }
}