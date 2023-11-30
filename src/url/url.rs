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
// Created Date: 2023/08/29 10:32:46

use std::fmt::Display;

use crate::{WebResult, peek, expect, next, WebError, Helper, Binary, Buf, Scheme, UrlError };



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
    pub const DEFAULT_PATH: &str = "/";

    pub fn new() -> Url {
        Url { scheme: Scheme::None, path: Self::DEFAULT_PATH.to_string(), username: None, password: None, domain: None, port: None, query: None }
    }

    #[inline]
    pub fn merge(&mut self, other: Url) {
        if other.scheme != Scheme::None && self.scheme != other.scheme {
            self.scheme = other.scheme;
        }
        if other.path != Self::DEFAULT_PATH  && self.path != other.path {
            self.path = other.path;
        }
        if other.username != None  && self.username != other.username {
            self.username = other.username;
        }
        if other.password != None  && self.password != other.password {
            self.password = other.password;
        }
        if other.domain != None  && self.domain != other.domain {
            self.domain = other.domain;
        }
        if other.port != None && other.port != Some(0) && self.port != other.port {
            self.port = other.port;
        }
        if other.query != None  && self.query != other.query {
            self.query = other.query;
        }
        
    }
    
    fn parse_url_token<'a>(buffer: &'a mut Binary, can_convert: bool) -> WebResult<Option<String>> {
        let mut result = Vec::with_capacity(buffer.len());
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
                    return Err(WebError::from(UrlError::UrlInvalid));
                }
                let t = Helper::convert_hex(next!(buffer)?);
                let u = Helper::convert_hex(next!(buffer)?);
                if t.is_none() || u.is_none() {
                    return Err(WebError::from(UrlError::UrlInvalid));
                }
                result.push(t.unwrap() * 16 + u.unwrap());
            } else {
                result.push(b);
            }
        }
        match String::from_utf8(result) {
            Ok(s) => Ok(Some(s)),
            Err(_) => Err(WebError::from(UrlError::UrlInvalid))
        }
    }

    pub fn parse(url: Vec<u8>) -> WebResult<Url> {
        let mut buffer = Binary::from(url);
        let mut b = peek!(buffer)?;
        let mut scheme = Scheme::None;
        // let mut scheme_end = None;
        let mut username = None;
        let mut password = None;
        let mut domain = None;
        let mut port = None;
        let mut path = None;
        let mut query: Option<_> = None;
        let mut is_first_slash = false;
        let mut has_domain = true;
        if Helper::is_alpha(b) {
            scheme = Scheme::parse_scheme(&mut buffer)?;
            expect!(buffer.next() == b':' => Err(WebError::from(UrlError::UrlInvalid)));
            expect!(buffer.next() == b'/' => Err(WebError::from(UrlError::UrlInvalid)));
            expect!(buffer.next() == b'/' => Err(WebError::from(UrlError::UrlInvalid)));
            buffer.mark_commit();
        } else if b == b'/' {
            is_first_slash = true;
            has_domain = false;
        } else {
            return Err(WebError::from(UrlError::UrlInvalid));
        }
        
        let check_func = Helper::is_token;

        loop {
            b = match peek!(buffer) {
                Ok(v) => v,
                Err(_) => {
                    if path.is_some() {
                        query = Some(buffer.clone_slice());
                    } else if domain.is_some() {
                        if !is_first_slash {
                            port = Some(buffer.clone_slice());
                        } else {
                            path = Some(buffer.clone_slice());
                        }
                    } else if domain.is_none() {
                        if has_domain {
                            domain = Some(buffer.clone_slice());
                        } else {
                            path = Some(buffer.clone_slice());
                        }
                    }
                    break;
                }
            };

            // 存在用户名, 解析用户名
            if b == b':' {
                //未存在协议头, 允许path与query, 忽略掉冒号
                if !is_first_slash {
                    // 匹配域名, 如果在存在期间检测到@则把当前当作用户结尾
                    if domain.is_none() {
                        domain = Some(buffer.clone_slice());
                    } else {
                        return Err(WebError::from(UrlError::UrlInvalid));
                    }
                    buffer.mark_bump();
                }
            } else if b == b'@' {
                //一开始的冒泡匹配域名,把域名结束当前username结束, 不存在用户密码, 不允许存在'@'
                if domain.is_none() {
                    return Err(WebError::from(UrlError::UrlInvalid));
                }
                username = domain;
                domain = None;
                password = Some(buffer.clone_slice());
                buffer.mark_bump();
            } else if b == b'/' {
                if !is_first_slash {
                    //反斜杠仅存在第一次域名不解析时获取
                    if domain.is_none() {
                        domain = Some(buffer.clone_slice());
                    } else {
                        port = Some(buffer.clone_slice());
                    }
                    is_first_slash = true;
                }
            } else if b == b'?' {
                if !is_first_slash {
                    if domain.is_none() && has_domain {
                        domain = Some(buffer.clone_slice());
                    }
                }
                // 多个'?'忽略当作query
                if path.is_none() {
                    path = Some(buffer.clone_slice());
                    buffer.mark_bump();
                }
            } else if !check_func(b) {
                return Err(WebError::from(UrlError::UrlInvalid));
            }

            next!(buffer)?;
        }

        let mut url = Url::new();
        url.scheme = scheme;
        if username.is_some() {
            url.username = Self::parse_url_token(&mut username.unwrap(), true)?;
        }
        if password.is_some() {
            url.password = Self::parse_url_token(&mut password.unwrap(), true)?;
        }
        if domain.is_some() {
            url.domain = Self::parse_url_token(&mut domain.unwrap(), true)?;
        }
        if port.is_some() {
            let port = Self::parse_url_token(&mut port.unwrap(), true)?;
            if port.is_some() {
                url.port = match port.unwrap().parse::<u16>() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(WebError::from(UrlError::UrlInvalid)),
                }
            }
        }
        
        if path.is_some() {
            url.path = Self::parse_url_token(&mut path.unwrap(), true)?.unwrap_or("/".to_string());
        }
        
        if query.is_some() {
            url.query = Self::parse_url_token(&mut query.unwrap(), true)?;
        }

        if url.port.is_none() {
            match url.scheme {
                Scheme::Http => url.port = Some(80),
                Scheme::Https => url.port = Some(443),
                Scheme::Ws => url.port = Some(80),
                Scheme::Wss => url.port = Some(443),
                Scheme::Ftp => url.port = Some(21),
                _ => url.port = Some(0),
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
                    return Err(WebError::from(UrlError::UrlCodeInvalid));
                }
                
                let t = Helper::convert_hex(bytes[idx + 1]);
                let u = Helper::convert_hex(bytes[idx + 2]);
                if t.is_none() || u.is_none() {
                    return Err(WebError::from(UrlError::UrlCodeInvalid));
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

    pub fn get_authority(&self) -> String {
        let port = if self.scheme != Scheme::None && self.port.is_some() {
            match (&self.scheme, self.port) {
                (Scheme::Http, Some(80)) => None,
                (Scheme::Https, Some(443)) => None,
                _ => Some(format!(":{}", self.port.as_ref().unwrap()).to_string())
            }
        } else {
            None
        };
        if self.domain.is_some() {
            format!("{}{}", self.domain.as_ref().unwrap(), port.unwrap_or(String::new()))
        } else {
            String::new()
        }
    }
    
    pub fn get_scheme(&self) -> String {
        self.scheme.as_str().to_string()
    }

    pub fn get_connect_url(&self) -> Option<String> {
        if self.domain.is_some() && self.port.is_some() {
            Some(format!(
                "{}:{}",
                self.domain.as_ref().unwrap(),
                self.port.as_ref().unwrap()
            ))
        } else {
            None
        }
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
        if self.scheme != Scheme::None && self.port.is_some() {
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

impl TryFrom<&[u8]> for Url {
    type Error=WebError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Url::parse(value.to_vec())
    }
}

impl TryFrom<&str> for Url {
    type Error=WebError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Url::parse(value.as_bytes().to_vec())
    }
}

impl TryFrom<String> for Url {
    type Error=WebError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Url::parse(value.into_bytes().to_vec())
    }
}

impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
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


mod tests {
    
    
    
    macro_rules! murl {
        ($name:ident, $buf:expr, |$arg:ident| $body:expr) => (
        #[test]
        fn $name() {
            let url = crate::Url::try_from($buf).unwrap();
            closure(url);
            fn closure($arg: crate::Url) {
                $body
            }
        }
        )
    }

        
    murl! {
        urltest_001,
        "https://%4811:!%2011@www.baidu.com:88/path?aaa=222",
        |u| {
            assert_eq!(u.scheme, crate::Scheme::Https);
            assert_eq!(u.domain.unwrap(), "www.baidu.com");
            assert_eq!(u.username.unwrap(), "H11");
            assert_eq!(u.password.unwrap(), "! 11");
            assert_eq!(u.port.unwrap(), 88);
            assert_eq!(u.path, "/path");
            assert_eq!(u.query.unwrap(), "aaa=222");
        }
    }
    
    murl! {
        urltest_002,
        "/path?aaa=222",
        |u| {
            assert_eq!(u.scheme, crate::Scheme::None);
            assert_eq!(u.domain, None);
            assert_eq!(u.username, None);
            assert_eq!(u.password, None);
            assert_eq!(u.path, "/path");
            assert_eq!(u.query.unwrap(), "aaa=222");
        }
    }

    murl! {
        urltest_003,
        "http://11:11@www.baidu.com/path",
        |u| {
            assert_eq!(u.scheme, crate::Scheme::Http);
            assert_eq!(u.domain.unwrap(), "www.baidu.com");
            assert_eq!(u.username.unwrap(), "11");
            assert_eq!(u.password.unwrap(), "11");
            assert_eq!(u.port.unwrap(), 80);
            assert_eq!(u.path, "/path");
            assert_eq!(u.query, None);
        }
    }


    murl! {
        urltest_003,
        "http://127.0.0.1:8080",
        |u| {
            assert_eq!(u.scheme, crate::Scheme::Http);
            assert_eq!(u.domain.unwrap(), "127.0.0.1");
            assert_eq!(u.port.unwrap(), 8080);
            assert_eq!(u.path, "/");
            assert_eq!(u.query, None);
        }
    }

}