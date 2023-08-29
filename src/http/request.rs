use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    io::Write,
    rc::Rc,
    sync::{Arc, RwLock},
};

use super::{
    http2::{self, encoder::Encoder, Decoder, HeaderIndex},
    HeaderMap, Method, Version,
};
use crate::{BinaryMut, Buf, BufMut, Extensions, HeaderName, HeaderValue, Helper, MarkBuf,
    Scheme, Serialize, Url, WebError, WebResult,
};

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
    pub path: String,
    pub extensions: Extensions,
}

#[derive(Debug)]
pub struct Builder {
    inner: WebResult<Parts>,
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct a `Request`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::http::request::Builder;
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

    pub fn from_req<T: Serialize>(req: Request<T>) -> Builder {
        let mut build = Builder::default();
        if req.method() != &Method::None {
            let _ = build.inner.as_mut().map(|head| {
                head.method = req.method().clone();
            });
        }
        if req.version() != &Version::None {
            let _ = build.inner.as_mut().map(|head| {
                head.version = req.version().clone();
            });
        }
        if req.path() != &Url::DEFAULT_PATH {
            let _ = build.inner.as_mut().map(|head| {
                head.path = req.path().clone();
            });
        }

        let _ = build.inner.as_mut().map(|head| {
            head.url = req.url().clone();
        });

        match req.extensions().get::<Arc<RwLock<HeaderIndex>>>() {
            Some(index) => {
                let _ = build.inner.as_mut().map(|head| {
                    head.extensions.insert(index.clone());
                });
            }
            _ => (),
        }
        build
    }

    /// Set the HTTP method for this request.
    ///
    /// By default this is `GET`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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

    /// Get the HTTP Method for this request.
    ///
    /// By default this is `GET`. If builder has error, returns None.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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

    /// Set the URI for this request.
    ///
    /// By default this is `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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
            Ok(head)
        })
    }

    /// Get the URI for this request
    ///
    /// By default this is `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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

    /// Set the HTTP version for this request.
    ///
    /// By default this is HTTP/1.1
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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

    /// Get the HTTP version for this request
    ///
    /// By default this is HTTP/1.1.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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

    /// Appends a header to this request builder.
    ///
    /// This function will append the provided key/value as a header to the
    /// internal `HeaderMap` being constructed. Essentially this is equivalent
    /// to calling `HeaderMap::append`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// # use webparse::header::HeaderValue;
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
            // let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
            // let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
            head.header.insert(key, value);
            Ok(head)
        })
    }

    /// Get header on this request builder.
    /// when builder has error returns None
    ///
    /// # Example
    ///
    /// ```
    /// # use webparse::Request;
    /// let req = Request::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( &headers["Accept"], "text/html" );
    /// assert_eq!( &headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_ref(&self) -> Option<&HeaderMap> {
        self.inner.as_ref().ok().map(|h| &h.header)
    }

    /// Get headers on this request builder.
    ///
    /// When builder has error returns None.
    ///
    /// # Example
    ///
    /// ```
    /// # use webparse::{HeaderValue, Request};
    /// let mut req = Request::builder();
    /// {
    ///   let headers = req.headers_mut().unwrap();
    ///   headers.insert("Accept", "text/html");
    ///   headers.insert("X-Custom-Foo", "bar");
    /// }
    /// let headers = req.headers_ref().unwrap();
    /// assert_eq!( &headers["Accept"], "text/html" );
    /// assert_eq!( &headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap> {
        self.inner.as_mut().ok().map(|h| &mut h.header)
    }

    /// "Consumes" this builder, using the provided `body` to return a
    /// constructed `Request`.
    ///
    /// # Errors
    ///
    /// This function may return an error if any previously configured argument
    /// failed to parse or get converted to the internal representation. For
    /// example if an invalid `head` was specified via `header("Foo",
    /// "Bar\r\n")` the error will be returned when this function is called
    /// rather than when `header` was called.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
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
            if head.path.len() == 0 {
                head.path = head.url.path.clone();
            }
            Request {
                parts: head,
                body,
                partial: true,
            }
        })
    }

    // private

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

    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<T> Request<T>
where
    T: Serialize,
{
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    #[inline]
    pub fn version(&self) -> &Version {
        &self.parts.version
    }

    #[inline]
    pub fn path(&self) -> &String {
        &self.parts.path
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.parts.header
    }

    pub fn scheme(&self) -> &Scheme {
        &self.parts.url.scheme
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.parts.header
    }

    pub fn url(&self) -> &Url {
        &self.parts.url
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

    pub fn parse_http2<B: Buf + MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let mut decoder = self.get_decoder();
        let headers = decoder.decode(buffer)?;
        for h in headers {
            if h.0.is_spec() {
                let value: String = (&h.1).try_into()?;
                match h.0.name() {
                    ":authority" => {
                        self.parts.url.domain = Some(value);
                        self.headers_mut().insert_exact(h.0, h.1);
                    }
                    ":method" => {
                        self.parts.method = Method::try_from(&*value)?;
                    }
                    ":path" => {
                        self.parts.path = value;
                    }
                    ":scheme" => {
                        self.parts.url.scheme = Scheme::try_from(&*value)?;
                    }
                    _ => {
                        self.headers_mut().insert_exact(h.0, h.1);
                    }
                }
            } else {
                self.headers_mut().insert_exact(h.0, h.1);
            }
        }
        if self.parts.path != "/".to_string() {
            let url = Url::parse(self.parts.path.as_bytes().to_vec())?;
            self.parts.url.merge(url);
        }
        self.parts.version = Version::Http2;
        Ok(buffer.mark_commit())
    }

    pub fn parse2(&mut self, buf: &[u8]) -> WebResult<usize> {
        self.partial = true;
        let mut buffer = BinaryMut::from(buf);
        self.parse_http2(&mut buffer)
    }

    pub fn parse_buffer<B: Buf + MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Helper::skip_empty_lines(buffer)?;
        {
            let chunks = buffer.chunk();
            if chunks.len() >= http2::HTTP2_MAGIC.len()
                && &chunks[..http2::HTTP2_MAGIC.len()] == http2::HTTP2_MAGIC
            {
                self.parts.version = Version::Http2;
                buffer.advance(http2::HTTP2_MAGIC.len());
                return self.parse_http2(buffer);
            }
        }

        self.parts.method = Helper::parse_method(buffer)?;
        Helper::skip_spaces(buffer)?;
        self.parts.path = Helper::parse_token(buffer)?.to_string();
        Helper::skip_spaces(buffer)?;
        self.parts.version = Helper::parse_version(buffer)?;
        Helper::skip_new_line(buffer)?;
        Helper::parse_header(buffer, &mut self.parts.header)?;
        self.partial = false;
        self.parts.url = match self.parts.method {
            // Connect 协议, Path则为连接地址,
            Method::Connect => {
                let mut url = Url::new();
                Self::parse_connect_by_host(&mut url, &self.parts.path)?;
                url
            }
            _ => {
                let mut url = Url::try_from(self.parts.path.to_string())?;
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
        };
        Ok(buffer.mark_commit())
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
    /// # use webparse::*;
    /// let request: Request<()> = Request::default();
    /// assert!(request.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    /// Returns a mutable reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let mut request: Request<()> = Request::default();
    /// request.extensions_mut().insert("hello");
    /// assert_eq!(request.extensions().get(), Some(&"hello"));
    /// ```
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    pub fn httpdata(&self) -> WebResult<Vec<u8>> {
        let mut buffer = BinaryMut::new();
        self.serialize(&mut buffer)?;
        return Ok(buffer.into_slice_all());
    }

    pub fn http2data(&mut self) -> WebResult<Vec<u8>> {
        let mut buffer = BinaryMut::new();
        self.serialize_mut(&mut buffer)?;
        return Ok(buffer.into_slice_all());
    }

    fn get_index(&mut self) -> Arc<RwLock<HeaderIndex>> {
        match self.extensions().get::<Arc<RwLock<HeaderIndex>>>() {
            Some(index) => index.clone(),
            None => {
                let index = Arc::new(RwLock::new(HeaderIndex::new()));
                self.extensions_mut().insert(index.clone());
                index
            }
        }
    }

    pub fn get_decoder(&mut self) -> Decoder {
        Decoder::new_index(self.get_index())
    }

    pub fn get_encoder(&mut self) -> Encoder {
        Encoder::new_index(self.get_index())
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

    // like wwww.baidu.com:80, wwww.google.com:443
    pub fn get_connect_url(&self) -> Option<String> {
        if self.url.domain.is_some() && self.url.port.is_some() {
            Some(format!(
                "{}:{}",
                self.url.domain.as_ref().unwrap(),
                self.url.port.as_ref().unwrap()
            ))
        } else {
            None
        }
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

impl Default for Parts {
    fn default() -> Self {
        Parts {
            method: Method::NONE,
            header: HeaderMap::new(),
            version: Version::Http11,
            url: Url::new(),
            path: String::new(),
            extensions: Extensions::new(),
        }
    }
}

impl<T> Serialize for Request<T>
where
    T: Serialize,
{
    fn serialize(&self, buffer: &mut BinaryMut) -> WebResult<()> {
        match self.parts.version {
            Version::Http11 => {
                self.parts.method.serialize(buffer)?;
                buffer.put_u8(b' ');
                self.parts.path.serialize(buffer)?;
                buffer.put_u8(b' ');
                self.parts.version.serialize(buffer)?;
                buffer.put_slice("\r\n".as_bytes());
                self.parts.header.serialize(buffer)?;
                self.body.serialize(buffer)?;
                Ok(())
            }
            Version::Http2 => Err(WebError::Extension(
                "http2 will may change dynamic header so so support",
            )),
            _ => Err(WebError::Extension("un support")),
        }
    }

    fn serialize_mut(&mut self, buffer: &mut BinaryMut) -> WebResult<()> {
        match self.parts.version {
            Version::Http2 => {
                let mut encode = self.get_encoder();
                encode.encode_header_into(
                    (
                        &HeaderName::from_static(":method"),
                        &HeaderValue::from_cow(self.parts.method.serial_bytes()?),
                    ),
                    buffer,
                )?;
                encode.encode_header_into(
                    (
                        &HeaderName::from_static(":path"),
                        &HeaderValue::from_cow(self.parts.path.serial_bytes()?),
                    ),
                    buffer,
                )?;
                if self.parts.url.scheme != Scheme::None {
                    encode.encode_header_into(
                        (
                            &HeaderName::from_static(":scheme"),
                            &HeaderValue::from_cow(self.parts.url.scheme.serial_bytes()?),
                        ),
                        buffer,
                    )?;
                }
                encode.encode_into(self.parts.header.iter(), buffer)?;
                self.body.serialize(buffer)?;
                Ok(())
            }
            _ => self.serialize(buffer),
        }
    }

    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Err(WebError::Serialize("request can't serial bytes"))
    }
}

mod tests {
    use crate::{http::request::Builder, Helper, Method, Request, Scheme, Version};

    macro_rules! req {
        ($name:ident, $buf:expr, |$arg:ident| $body:expr) => {
            #[test]
            fn $name() {
                let mut req = Request::new();
                let size = req.parse($buf.as_ref()).unwrap();
                assert_eq!(size, $buf.len());
                assert_eq!(&req.httpdata().unwrap(), $buf);
                closure(req);
                fn closure($arg: Request<()>) {
                    $body
                }
            }
        };
    }

    macro_rules! req2 {
        ($name:ident, $buf:expr, |$arg:ident| $body:expr) => {
            #[test]
            fn $name() {
                let mut req = Request::new();
                let size = req.parse2($buf.as_ref()).unwrap();
                assert_eq!(size, $buf.len());
                // assert_eq!(&req.httpdata().unwrap(), $buf);
                closure(req);
                fn closure($arg: Request<()>) {
                    $body
                }
            }
        };
    }

    req! {
        urltest_001,
        b"GET /bar;par?b HTTP/1.1\r\nHost: foo\r\n\r\n",
        |req| {
            assert_eq!(req.method(), &Method::Get);
            assert_eq!(req.path(), "/bar;par?b");
            assert_eq!(&req.url().path, "/bar;par");
            assert_eq!(req.url().query, Some("b".to_string()));
            assert_eq!(req.version(), &Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], "foo");
        }
    }

    req! {
        urltest_002,
        b"GET //:///// HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &Method::Get);
            assert_eq!(req.path(), "//://///");
            assert_eq!(&req.url().path, "//://///");
            assert_eq!(req.url().query, None);
            assert_eq!(req.version(), &Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], "");
        }
    }

    req! {
        urltest_003,
        b"GET /abcd?efgh?ijkl HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &Method::Get);
            assert_eq!(req.path(), "/abcd?efgh?ijkl");
            assert_eq!(&req.url().path, "/abcd");
            assert_eq!(req.url().query, Some("efgh?ijkl".to_string()));
            assert_eq!(req.version(), &Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], "");
        }
    }

    req! {
        urltest_004,
        b"GET /foo/[61:27]/:foo HTTP/1.1\r\nHost: \r\n\r\n",
        |req| {
            assert_eq!(req.method(), &Method::Get);
            assert_eq!(req.path(), "/foo/[61:27]/:foo");
            assert_eq!(&req.url().path, "/foo/[61:27]/:foo");
            assert_eq!(req.url().query, None);
            assert_eq!(req.version(), &Version::Http11);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()["Host"], "");
        }
    }

    req2! {
        urltest_005,
        Helper::hexstr_to_vec("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d"),
        |req| {
            assert_eq!(req.method(), &Method::Get);
            assert_eq!(req.path(), "/");
            assert_eq!(&req.url().path, "/");
            assert_eq!(req.url().query, None);
            assert_eq!(req.version(), &Version::Http2);
            assert_eq!(req.headers().len(), 1);
            assert_eq!(&req.headers()[":authority"], "www.example.com");
        }
    }

    #[test]
    fn http2_test() {
        let mut req = Request::new();
        let buf = Helper::hexstr_to_vec("8286 8441 0f77 7777 2e65 7861 6d70 6c65 2e63 6f6d");
        let size = req.parse2(buf.as_ref()).unwrap();
        assert_eq!(size, buf.len());
        assert_eq!(req.method(), &Method::Get);
        assert_eq!(req.scheme(), &Scheme::Http);
        assert_eq!(req.path(), "/");
        assert_eq!(&req.url().path, "/");
        assert_eq!(req.url().query, None);
        assert_eq!(req.version(), &Version::Http2);
        assert_eq!(req.headers().len(), 1);
        assert_eq!(&req.headers()[":authority"], "www.example.com");

        let mut req = Builder::from_req(req).body(()).unwrap();
        let buf = Helper::hexstr_to_vec("8286 84be 5808 6e6f 2d63 6163 6865");
        let size = req.parse2(buf.as_ref()).unwrap();
        assert_eq!(size, buf.len());

        assert_eq!(req.method(), &Method::Get);
        assert_eq!(req.scheme(), &Scheme::Http);
        assert_eq!(req.path(), "/");
        assert_eq!(&req.url().path, "/");
        assert_eq!(req.url().query, None);
        assert_eq!(req.version(), &Version::Http2);
        assert_eq!(req.headers().len(), 2);
        assert_eq!(&req.headers()[":authority"], "www.example.com");
        assert_eq!(&req.headers()["cache-control"], "no-cache");

        let mut req = Builder::from_req(req).body(()).unwrap();
        let buf = Helper::hexstr_to_vec(
            "8287 85bf 400a 6375 7374 6f6d 2d6b 6579 0c63 7573 746f 6d2d 7661 6c75 65",
        );
        let size = req.parse2(buf.as_ref()).unwrap();
        assert_eq!(size, buf.len());
        assert_eq!(req.method(), &Method::Get);
        assert_eq!(req.scheme(), &Scheme::Https);
        assert_eq!(req.path(), "/index.html");
        assert_eq!(&req.url().path, "/index.html");
        assert_eq!(req.url().query, None);
        assert_eq!(req.version(), &Version::Http2);
        assert_eq!(req.headers().len(), 2);
        assert_eq!(&req.headers()[":authority"], "www.example.com");
        assert_eq!(&req.headers()["custom-key"], "custom-value");
    }
}
