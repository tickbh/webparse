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
// Created Date: 2023/08/15 10:00:38

use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use super::{http2::HeaderIndex, HeaderMap, Method, Version};
use crate::{
    http2::frame::Settings, Extensions, HeaderName, HeaderValue, Helper, Scheme, Serialize, Url,
    WebError, WebResult,
};
use algorithm::buf::{BinaryMut, Bt, BtMut};

#[derive(Debug)]
pub struct Request<T>
where
    T: Serialize,
{
    parts: Parts,
    body: T,
    partial: bool,
}

#[derive(Debug)]
pub struct Parts {
    pub method: Method,
    pub header: HeaderMap,
    pub version: Version,
    pub url: Url,
    pub extensions: Extensions,
}

#[derive(Debug)]
pub struct Builder {
    inner: WebResult<Parts>,
}

impl Builder {
    /// 创建默认的Builder对象.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::http::request::Builder;
    ///
    /// let req = Builder::new()
    ///     .method("POST")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn new() -> Builder {
        Builder::default()
    }

    pub fn from_req<T: Serialize>(req: &Request<T>) -> Builder {
        let mut build = Builder::default();
        if req.method() != &Method::None {
            let _ = build.inner.as_mut().map(|head| {
                head.method = req.method().clone();
            });
        }
        if req.version() != Version::None {
            let _ = build.inner.as_mut().map(|head| {
                head.version = req.version().clone();
            });
        }
        if req.path() != &Url::DEFAULT_PATH {
            let _ = build.inner.as_mut().map(|head| {
                head.url.path = req.path().clone();
            });
        }

        let _ = build.inner.as_mut().map(|head| {
            head.url = req.url().clone();
        });

        build
    }

    /// 设置HTTP的方法, 默认值为'GET'
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let req = Request::builder()
    ///     .method("POST")
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn method<T>(self, method: T) -> Builder
    where
        Method: TryFrom<T>,
        <Method as TryFrom<T>>::Error: Into<WebError>,
    {
        self.and_then(move |mut head| {
            let method = TryFrom::try_from(method).map_err(Into::into)?;
            head.method = method;
            Ok(head)
        })
    }

    /// 获取HTTP的值, 如果Builder有错误返回空
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.method_ref(),Some(&Method::Get));
    ///
    /// req = req.method("POST");
    /// assert_eq!(req.method_ref(),Some(&Method::POST));
    /// ```
    pub fn method_ref(&self) -> Option<&Method> {
        self.inner.as_ref().ok().map(|h| &h.method)
    }

    /// 设置请求的URL,可包含http(s)前缀,或仅为'/'开头的路径
    ///
    /// 默认值为 `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let req = Request::builder()
    ///     .url("https://www.rust-lang.org/")
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn url<T>(self, url: T) -> Builder
    where
        Url: TryFrom<T>,
        <Url as TryFrom<T>>::Error: Into<WebError>,
    {
        self.and_then(move |mut head| {
            head.url = TryFrom::try_from(url).map_err(Into::into)?;
            if let Some(connect) = &head.url.get_connect_url() {
                head.header.insert("Host", connect.clone());
            }
            Ok(head)
        })
    }

    /// 获取Url的值, 若有错误返回None
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.url_ref().unwrap(), "/" );
    ///
    /// req = req.url("https://www.rust-lang.org/");
    /// assert_eq!(req.url_ref().unwrap(), "https://www.rust-lang.org/" );
    /// ```
    pub fn url_ref(&self) -> Option<&Url> {
        self.inner.as_ref().ok().map(|h| &h.url)
    }

    /// 设置http的版本格式, 默认为'HTTP/1.1'
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let req = Request::builder()
    ///     .version(Version::Http2)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn version(self, version: Version) -> Builder {
        self.and_then(move |mut head| {
            head.version = version;
            Ok(head)
        })
    }

    /// 获取http的版本格式
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let mut req = Request::builder();
    /// assert_eq!(req.version_ref().unwrap(), &Version::Http11 );
    ///
    /// req = req.version(Version::Http2);
    /// assert_eq!(req.version_ref().unwrap(), &Version::Http2 );
    /// ```
    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|h| &h.version)
    }

    /// 添加头文件信息到Builder底下, 当前header实现的是无序的HashMap格式
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let req = Request::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar")
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn header<K, V>(self, key: K, value: V) -> Builder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<WebError>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<WebError>,
    {
        self.and_then(move |mut head| {
            head.header.insert(key, value);
            Ok(head)
        })
    }

    /// 从另一个HeaderMap中进行header构建
    pub fn headers(self, header: HeaderMap) -> Builder {
        self.and_then(move |mut head| {
            for iter in header {
                head.header.insert(iter.0, iter.1);
            }
            Ok(head)
        })
    }

    /// 获取头信息的引用
    ///
    /// # Example
    ///
    /// ```
    /// use webparse::Request;
    /// let req = Request::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( &headers["Accept"], &"text/html" );
    /// assert_eq!( &headers["X-Custom-Foo"], &"bar" );
    /// ```
    pub fn headers_ref(&self) -> Option<&HeaderMap> {
        self.inner.as_ref().ok().map(|h| &h.header)
    }

    /// 获取可更改头的信息
    ///
    /// # Example
    ///
    /// ```
    /// use webparse::{HeaderValue, Request};
    /// let mut req = Request::builder();
    /// {
    ///   let headers = req.headers_mut().unwrap();
    ///   headers.insert("Accept", "text/html");
    ///   headers.insert("X-Custom-Foo", "bar");
    /// }
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( &headers["Accept"], &"text/html" );
    /// assert_eq!( &headers["X-Custom-Foo"], &"bar" );
    /// ```
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap> {
        self.inner.as_mut().ok().map(|h| &mut h.header)
    }

    /// 传入Body信息,构建出Request的请求信息
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    ///
    /// let request = Request::builder()
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn body<T>(self, body: T) -> WebResult<Request<T>>
    where
        T: Serialize,
    {
        self.inner.map(move |mut head| {
            let server = HeaderName::from_static("User-Agent");
            if !head.header.contains(&server) {
                head.header.insert(server, "wenmeng");
            }
            Request {
                parts: head,
                body,
                partial: true,
            }
        })
    }

    /// 获取请求的body长度, 如果为0则表示不存在长度信息,
    /// 直到收到关闭信息则表示结束, http/1.1为关闭链接, http/2则是end_stream
    pub fn get_body_len(&self) -> isize {
        if let Ok(inner) = &self.inner {
            inner.header.get_body_len()
        } else {
            0
        }
    }

    pub fn upgrade_http2(self, settings: Settings) -> Self {
        self.and_then(move |mut head| {
            head.header.insert("Connection", "Upgrade, HTTP2-Settings");
            head.header.insert("Upgrade", "h2c");
            head.header
                .insert("HTTP2-Settings", settings.encode_http_settings());
            Ok(head)
        })
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Parts) -> WebResult<Parts>,
    {
        Builder {
            inner: self.inner.and_then(func),
        }
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Builder {
        let mut parts = Parts::new();
        parts.method = Method::Get;
        Builder { inner: Ok(parts) }
    }
}

impl Request<()> {
    pub fn new() -> Request<()> {
        Request {
            body: (),
            partial: false,
            parts: Parts::new(),
        }
    }

    pub fn new_by_parts(parts: Parts) -> Request<()> {
        Request {
            body: (),
            partial: false,
            parts,
        }
    }

    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<T> Request<T>
where
    T: Serialize,
{
    /// 查看请求是否是http2
    pub fn is_http2(&self) -> bool {
        self.parts.version == Version::Http2
    }

    pub fn set_url(&mut self, url: Url) {
        if let Some(connect) = url.get_connect_url() {
            if !self.headers().contains(&"Host") {
                self.headers_mut().insert("Host", connect.clone());
            }
        }

        self.parts.url = url;
    }

    /// 返回parts信息
    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    /// 返回parts信息
    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    #[inline]
    pub fn set_method(&mut self, method: Method) {
        self.parts.method = method;
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.parts.version
    }

    #[inline]
    pub fn set_version(&mut self, version: Version) {
        self.parts.version = version;
    }

    #[inline]
    pub fn path(&self) -> &String {
        &self.parts.url.path
    }

    #[inline]
    pub fn set_path(&mut self, path: String) {
        self.parts.url.path = path;
    }

    pub fn scheme(&self) -> &Scheme {
        &self.parts.url.scheme
    }

    #[inline]
    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.parts.url.scheme = scheme;
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.parts.header
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.parts.header
    }

    pub fn headers_body(&mut self) -> (&HeaderMap, &T) {
        (&self.parts.header, &self.body)
    }

    pub fn headers_body_mut(&mut self) -> (&mut HeaderMap, &mut T) {
        (&mut self.parts.header, &mut self.body)
    }

    pub fn url(&self) -> &Url {
        &self.parts.url
    }
    
    pub fn url_mut(&mut self) -> &mut Url {
        &mut self.parts.url
    }

    pub fn get_host(&self) -> Option<String> {
        self.parts.get_host()
    }

    pub fn get_referer(&self) -> Option<String> {
        self.parts.get_referer()
    }

    pub fn get_user_agent(&self) -> Option<String> {
        self.parts.get_user_agent()
    }

    pub fn get_cookie(&self) -> Option<String> {
        self.parts.get_cookie()
    }

    /// 返回完整的域名加上端口号信息
    /// 如wwww.baidu.com:80, wwww.google.com:443
    pub fn get_connect_url(&self) -> Option<String> {
        self.parts.get_connect_url()
    }

    /// 获取请求的body长度, 如果为0则表示不存在长度信息,
    /// 直到收到关闭信息则表示结束, http/1.1为关闭链接, http/2则是end_stream
    pub fn get_body_len(&self) -> isize {
        self.parts.header.get_body_len()
    }

    /// 获取请求的authority
    pub fn get_authority(&self) -> String {
        self.parts.url.get_authority()
    }

    /// 获取请求的scheme
    pub fn get_scheme(&self) -> String {
        self.parts.url.get_scheme()
    }

    /// 是否保持心跳活跃
    pub fn is_keep_alive(&self) -> bool {
        self.parts.header.is_keep_alive()
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    pub fn is_complete(&self) -> bool {
        !self.partial
    }

    pub fn into<B: Serialize>(self, body: B) -> (Request<B>, T) {
        let new = Request {
            body,
            parts: self.parts,
            partial: self.partial,
        };
        (new, self.body)
    }

    pub fn into_type<B: From<T> + Serialize>(self) -> Request<B> {
        let new = Request {
            body: From::from(self.body),
            parts: self.parts,
            partial: self.partial,
        };
        new
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

    pub fn parse_buffer<B: Bt>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let len = buffer.remaining();
        self.partial = true;
        Helper::skip_empty_lines(buffer)?;
        self.parts.method = Helper::parse_method(buffer)?;
        Helper::skip_spaces(buffer)?;
        let path = Helper::parse_token(buffer)?.to_string();
        Helper::skip_spaces(buffer)?;
        self.parts.version = Helper::parse_version(buffer)?;
        Helper::skip_new_line(buffer)?;
        Helper::parse_header(buffer, &mut self.parts.header)?;
        self.partial = false;
        self.parts.url = match self.parts.method {
            // Connect 协议, Path则为连接地址,
            Method::Connect => {
                let mut url = Url::new();
                Self::parse_connect_by_host(&mut url, &path)?;
                url
            }
            _ => {
                let mut url = Url::try_from(path)?;
                if url.domain.is_none() {
                    match self.parts.header.get_host() {
                        Some(h) => {
                            Self::parse_connect_by_host(&mut url, &h)?;
                        }
                        _ => (),
                    }
                }

                if url.scheme.is_none() {
                    match self.parts.header.get_option_value(&":scheme") {
                        Some(h) => {
                            url.scheme = TryFrom::try_from(&*h.to_string())
                                .ok()
                                .unwrap_or(Scheme::Http);
                        }
                        _ => {
                            url.scheme = Scheme::Http;
                        }
                    }
                }
                url
            }
        };
        Ok(len - buffer.remaining())
    }

    pub fn parse(&mut self, buf: &[u8]) -> WebResult<usize> {
        self.partial = true;
        let mut buffer = BinaryMut::from(buf);
        self.parse_buffer(&mut buffer)
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::*;
    /// let request: Request<()> = Request::default();
    /// assert!(request.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    pub fn http1_data(&mut self) -> WebResult<Vec<u8>> {
        let mut buffer = BinaryMut::new();
        self.encode_header(&mut buffer)?;
        self.body.serialize(&mut buffer)?;
        return Ok(buffer.into_slice_all());
    }

    pub fn body(&self) -> &T {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    pub fn encode_header<B: Bt + BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += self.parts.method.encode(buffer)?;
        size += buffer.put_u8(b' ');
        size += self.parts.url.path.serialize(buffer)?;
        size += buffer.put_u8(b' ');
        size += self.parts.version.encode(buffer)?;
        size += buffer.put_slice("\r\n".as_bytes());
        size += self.parts.header.encode(buffer)?;
        Ok(size)
    }

    pub fn replace_clone(&mut self, mut body: T) -> Request<T> {
        let parts = self.parts.clone();
        let partial = self.partial;
        std::mem::swap(&mut self.body, &mut body);
        Request {
            parts,
            body,
            partial,
        }
    }
}

impl Parts {
    pub fn new() -> Parts {
        Parts::default()
    }

    pub fn get_host(&self) -> Option<String> {
        if self.url.domain.is_some() {
            return self.url.domain.clone();
        }
        self.header.get_host()
    }

    pub fn get_referer(&self) -> Option<String> {
        self.header.get_referer()
    }

    pub fn get_user_agent(&self) -> Option<String> {
        self.header.get_user_agent()
    }

    pub fn get_cookie(&self) -> Option<String> {
        self.header.get_cookie()
    }

    // like wwww.baidu.com:80, wwww.google.com:443
    pub fn get_connect_url(&self) -> Option<String> {
        self.url.get_connect_url()
    }
}

impl Default for Request<()> {
    fn default() -> Self {
        Self {
            parts: Default::default(),
            body: Default::default(),
            partial: Default::default(),
        }
    }
}

impl<T> Display for Request<T>
where
    T: Serialize + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.parts.method.fmt(f)?;
        f.write_str(" ")?;
        self.parts.url.path.fmt(f)?;
        f.write_str(" ")?;
        self.parts.version.fmt(f)?;
        f.write_str("\r\n")?;
        self.parts.header.fmt(f)?;
        self.body.fmt(f)
    }
}

impl Default for Parts {
    fn default() -> Self {
        Parts {
            method: Method::NONE,
            header: HeaderMap::new(),
            version: Version::Http11,
            url: Url::new(),
            extensions: Extensions::new(),
        }
    }
}

impl Clone for Parts {
    fn clone(&self) -> Self {
        let mut value = Self {
            method: self.method.clone(),
            header: self.header.clone(),
            version: self.version.clone(),
            url: self.url.clone(),
            extensions: Extensions::new(),
        };

        match self.extensions.get::<Arc<RwLock<HeaderIndex>>>() {
            Some(index) => {
                value.extensions.insert(index.clone());
            }
            _ => (),
        }
        value
    }
}

#[cfg(test)]
mod tests {

    macro_rules! req {
        ($name:ident, $buf:expr, |$arg:ident| $body:expr) => {
            #[test]
            fn $name() {
                let mut req = crate::Request::new();
                let size = req.parse($buf.as_ref()).unwrap();
                assert_eq!(size, $buf.len());
                assert_eq!(&req.http1_data().unwrap(), $buf);
                closure(req);
                fn closure($arg: crate::Request<()>) {
                    $body
                }
            }
        };
    }

    req! {
        urltest_001,
        b"GET /bar;par?b HTTP/1.1\r\nHost: foo\r\n\r\n",
        |req| {
            assert_eq!(req.method(), &crate::Method::Get);
            assert_eq!(req.path(), "/bar;par?b");
            assert_eq!(&req.url().path, "/bar;par");
            assert_eq!(req.url().query, Some("b".to_string()));
            assert_eq!(req.version(), crate::Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], &"foo");
        }
    }

    req! {
        urltest_002,
        b"GET //:///// HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &crate::Method::Get);
            assert_eq!(req.path(), "//://///");
            assert_eq!(&req.url().path, "//://///");
            assert_eq!(req.url().query, None);
            assert_eq!(req.version(), crate::Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], &"");
        }
    }

    req! {
        urltest_003,
        b"GET /abcd?efgh?ijkl HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &crate::Method::Get);
            assert_eq!(req.path(), "/abcd?efgh?ijkl");
            assert_eq!(&req.url().path, "/abcd");
            assert_eq!(req.url().query, Some("efgh?ijkl".to_string()));
            assert_eq!(req.version(), crate::Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], &"");
        }
    }

    req! {
        urltest_004,
        b"GET /foo/[61:27]/:foo HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &crate::Method::Get);
            assert_eq!(req.path(), "/foo/[61:27]/:foo");
            assert_eq!(&req.url().path, "/foo/[61:27]/:foo");
            assert_eq!(req.url().query, None);
            assert_eq!(req.version(), crate::Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], &"");
        }
    }

    // req2! {
    //     urltest_005,
    //     Helper::hex_to_vec("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d"),
    //     |req| {
    //         assert_eq!(req.method(), &Method::Get);
    //         assert_eq!(req.path(), "/");
    //         assert_eq!(&req.url().path, "/");
    //         assert_eq!(req.url().query, None);
    //         assert_eq!(req.version(), &Version::Http2);
    //         assert_eq!(req.headers().len(), 1);
    //         assert_eq!(&req.headers()[":authority"], "www.example.com");
    //     }
    // }

    // #[test]
    // fn http2_test() {
    //     let mut req = Request::new();
    //     let buf = Helper::hex_to_vec("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d");
    //     let size = req.parse2(buf.as_ref()).unwrap();
    //     assert_eq!(size, buf.len());
    //     assert_eq!(req.method(), &Method::Get);
    //     assert_eq!(req.scheme(), &Scheme::Http);
    //     assert_eq!(req.path(), "/");
    //     assert_eq!(&req.url().path, "/");
    //     assert_eq!(req.url().query, None);
    //     assert_eq!(req.version(), Version::Http2);
    //     assert_eq!(req.headers().len(), 1);
    //     assert_eq!(&req.headers()[":authority"], "www.example.com");

    //     let mut req = Builder::from_req(&req).body(()).unwrap();
    //     let buf = Helper::hex_to_vec("8286 84be 5808 6e6f 2d63 6163 6865");
    //     let size = req.parse2(buf.as_ref()).unwrap();
    //     assert_eq!(size, buf.len());

    //     assert_eq!(req.method(), &Method::Get);
    //     assert_eq!(req.scheme(), &Scheme::Http);
    //     assert_eq!(req.path(), "/");
    //     assert_eq!(&req.url().path, "/");
    //     assert_eq!(req.url().query, None);
    //     assert_eq!(req.version(), Version::Http2);
    //     assert_eq!(req.headers().len(), 2);
    //     assert_eq!(&req.headers()[":authority"], "www.example.com");
    //     assert_eq!(&req.headers()["cache-control"], "no-cache");

    //     let mut req = Builder::from_req(&req).body(()).unwrap();
    //     let buf = Helper::hex_to_vec(
    //         "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65",
    //     );
    //     let size = req.parse2(buf.as_ref()).unwrap();
    //     assert_eq!(size, buf.len());
    //     assert_eq!(req.method(), &Method::Get);
    //     assert_eq!(req.scheme(), &Scheme::Https);
    //     assert_eq!(req.path(), "/index.html");
    //     assert_eq!(&req.url().path, "/index.html");
    //     assert_eq!(req.url().query, None);
    //     assert_eq!(req.version(), Version::Http2);
    //     assert_eq!(req.headers().len(), 2);
    //     assert_eq!(&req.headers()[":authority"], "www.example.com");
    //     assert_eq!(&req.headers()["custom-key"], "custom-value");
    // }
}
