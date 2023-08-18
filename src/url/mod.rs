
mod scheme;
mod builder;

use std::fmt::Display;

pub use scheme::Scheme;
pub use builder::Builder;

use crate::{WebResult, Buffer, peek, expect, next, WebError, Helper};



#[derive(Clone, Debug)]
pub struct Url {
    pub scheme: Scheme,
    pub path: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub query: Option<String>,
}

impl Url {
    
    pub fn new() -> Url {
        Url { scheme: Scheme::None, path: "/".to_string(), username: None, password: None, domain: None, port: None, query: None }
    }

    fn parse_url_token<'a>(buffer: &'a mut Buffer, start: usize, end: usize, can_convert: bool) -> WebResult<Option<String>> {
        if start >= end {
            return Ok(None)
        }
        let ori_start = buffer.get_start();
        let ori_cursor = buffer.get_cursor();
        let ori_end = buffer.get_end();
        let mut result = Vec::with_capacity(end-start);
        buffer.set_start(start);
        buffer.set_cursor(start);
        buffer.set_end(end);
        loop {
            let b = match next!(buffer) {
                Ok(v) => v,
                Err(_) => {
                    break;
                }
            };
            // 转码字符, 后面必须跟两位十六进制数字
            if b == b'%' {
                if !can_convert {
                    return Err(WebError::UrlInvalid);
                }
                let t = Helper::convert_hex(next!(buffer)?);
                let u = Helper::convert_hex(next!(buffer)?);
                if t.is_none() || u.is_none() {
                    return Err(WebError::UrlInvalid);
                }
                result.push(t.unwrap() * 16 + u.unwrap());
            } else {
                result.push(b);
            }
        }
        buffer.set_start(ori_start);
        buffer.set_cursor(ori_cursor);
        buffer.set_end(ori_end);
        match String::from_utf8(result) {
            Ok(s) => Ok(Some(s)),
            Err(_) => Err(WebError::UrlInvalid)
        }
    }

    pub fn parse(url: &str) -> WebResult<Url> {
        let mut buffer = Buffer::new_buf(url.as_bytes());
        let mut b = peek!(buffer)?;
        let mut scheme = Scheme::None;
        let mut scheme_end = 0;
        let mut username_end = 0;
        let mut password_end = 0;
        let mut domain_end = 0;
        let mut port_end = 0;
        let mut path_end = 0;
        let mut query_end = 0;
        let mut is_first_slash = false;
        let mut has_domain = true;
        if Helper::is_alpha(b) {
            scheme = Scheme::parse_scheme(&mut buffer)?;
            scheme_end = buffer.get_cursor();
            expect!(buffer.next() == b':' => Err(WebError::UrlInvalid));
            expect!(buffer.next() == b'/' => Err(WebError::UrlInvalid));
            expect!(buffer.next() == b'/' => Err(WebError::UrlInvalid));
        } else if b == b'/' {
            is_first_slash = true;
            has_domain = false;
        } else {
            return Err(WebError::UrlInvalid);
        }
        
        let mut check_func = Helper::is_token;

        loop {
            b = match next!(buffer) {
                Ok(v) => v,
                Err(_) => {
                    if path_end != 0 {
                        query_end = buffer.get_end();
                    } else if domain_end != 0 {
                        path_end = buffer.get_end();
                    } else if domain_end == 0 {
                        if has_domain {
                            domain_end = buffer.get_end();
                        } else {
                            path_end = buffer.get_end();
                        }
                    }
                    break;
                }
            };
            // 存在用户名, 解析用户名
            if b == b':' {
                //未存在协议头, 允许path与query
                if scheme_end == 0 || is_first_slash {
                    return Err(WebError::UrlInvalid);
                }

                // 匹配域名, 如果在存在期间检测到@则把当前当作用户结尾
                if domain_end == 0 {
                    domain_end = buffer.get_cursor() - 1;
                } else {
                    return Err(WebError::UrlInvalid);
                }

            } else if b == b'@' {
                //一开始的冒泡匹配域名,把域名结束当前username结束, 不存在用户密码, 不允许存在'@'
                if domain_end == 0 {
                    return Err(WebError::UrlInvalid);
                }
                username_end = domain_end;
                domain_end = 0;
                password_end = buffer.get_cursor() - 1;
            } else if b == b'/' {
                if !is_first_slash {
                    //反斜杠仅存在第一次域名不解析时获取
                    if domain_end == 0 {
                        domain_end = buffer.get_cursor() - 1;
                    } else {
                        port_end = buffer.get_cursor() - 1;
                    }
                    is_first_slash = true;
                }
            } else if b == b'?' {
                if domain_end == 0 && has_domain {
                    domain_end = buffer.get_cursor() - 1;
                }
                // 不允许存在多次的'?'
                if path_end != 0 {
                    return Err(WebError::UrlInvalid);
                }
                path_end = buffer.get_cursor() - 1;
            } else if !check_func(b) {
                return Err(WebError::UrlInvalid);
            }
        }

        let mut url = Url::new();
        url.scheme = scheme;
        if username_end != 0 {
            url.username = Self::parse_url_token(&mut buffer, scheme_end + 3, username_end, true)?;
        }
        if password_end != 0 {
            url.password = Self::parse_url_token(&mut buffer, username_end + 1, password_end, true)?;
        }
        if domain_end != 0 {
            url.domain = Self::parse_url_token(&mut buffer, std::cmp::max(password_end + 1, scheme_end + 3), domain_end, true)?;
        }
        if port_end != 0 {
            let port = Self::parse_url_token(&mut buffer, domain_end + 1, port_end, true)?;
            if port.is_some() {
                url.port = match port.unwrap().parse::<u16>() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(WebError::UrlInvalid),
                }
            }
        }
        if path_end != 0 {
            let mut path_start = 0;
            if domain_end != 0 {
                path_start = std::cmp::max(domain_end, port_end);
            }
            url.path = Self::parse_url_token(&mut buffer, path_start, path_end, true)?.unwrap_or("/".to_string());
        }
        
        if query_end != 0 {
            url.query = Self::parse_url_token(&mut buffer, path_end + 1, query_end, true)?;
        }

        if url.port.is_none() {
            match url.scheme {
                Scheme::Http => url.port = Some(80),
                Scheme::Https => url.port = Some(443),
                Scheme::Ws => url.port = Some(80),
                Scheme::Wss => url.port = Some(443),
                Scheme::Ftp => url.port = Some(21),
                _ => return Err(WebError::UrlInvalid)
            }
        }

        Ok(url)
    }

    pub fn url_encode(val: &str) -> String {
        let bytes = val.as_bytes();
        let mut vec = Vec::with_capacity((1.2 * (bytes.len() as f32)) as usize);
        for b in bytes {
            if Helper::is_not_uritrans(*b) {
                vec.push(*b);
            } else {
                vec.push(b'%');
                vec.push(Helper::to_hex((b / 16) as u8));
                vec.push(Helper::to_hex((b % 16) as u8));
            }
        }

        String::from_utf8_lossy(&vec).to_string()
    }
    
    pub fn url_decode(val: &str) -> WebResult<String> {
        let bytes = val.as_bytes();
        let mut vec = Vec::with_capacity(bytes.len() as usize);
        let mut idx = 0;
        loop {
            if idx >= bytes.len() {
                break;
            }
            let b = bytes[idx];
            if b == b'%' {
                if idx + 2 >= bytes.len() {
                    return Err(WebError::UrlCodeInvalid);
                }
                
                let t = Helper::convert_hex(bytes[idx + 1]);
                let u = Helper::convert_hex(bytes[idx + 2]);
                if t.is_none() || u.is_none() {
                    return Err(WebError::UrlCodeInvalid);
                }
                vec.push(t.unwrap() * 16 + u.unwrap());
                idx += 3;
            } else {
                vec.push(b);
                idx += 1;
            }
        }
        Ok(String::from_utf8_lossy(&vec).to_string())
    }
}

impl Display for Url {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.scheme != Scheme::None {
            f.write_fmt(format_args!("{}://", self.scheme))?;
        }
        if self.username.is_some() || self.password.is_some() {
            f.write_fmt(format_args!("{}:{}@", Self::url_encode(self.username.as_ref().unwrap_or(&String::new())) , Self::url_encode(self.password.as_ref().unwrap_or(&String::new()))))?;
        }
        if self.domain.is_some() {
            f.write_fmt(format_args!("{}", self.domain.as_ref().unwrap()))?;
        }
        if self.port.is_some() {
            match (&self.scheme, self.port) {
                (Scheme::Http, Some(80)) => {}
                (Scheme::Https, Some(443)) => {}
                _ => f.write_fmt(format_args!(":{}", self.port.as_ref().unwrap()))?
            };
        }
        f.write_fmt(format_args!("{}", Self::url_encode(&self.path)))?;
        if self.query.is_some() {
            f.write_fmt(format_args!("?{}", Self::url_encode(self.query.as_ref().unwrap())))?;
        }
        Ok(())
    }
}

impl TryFrom<&str> for Url {
    type Error=WebError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value)
    }
}

impl TryFrom<String> for Url {
    type Error=WebError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Url::parse(&value)
    }
}

impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
        println!("Ok === {}", format!("{}", &self));
        format!("{}", &self) == other
    }
}


impl PartialEq<Url> for str {
    fn eq(&self, url: &Url) -> bool {
        url == self
    }
}

impl Default for Url {
    #[inline]
    fn default() -> Url {
        Url::new()
    }
}
