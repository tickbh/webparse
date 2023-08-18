
use std::{collections::HashMap, io::Write};

use crate::{Buffer, WebResult, Url, Helper, WebError, HeaderName, HeaderValue, Extensions, Serialize};
use super::{Method, HeaderMap, Version};

#[derive(Debug)]
pub struct Request<T> 
where T : Serialize {
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
    /// # use webparse::*;
    ///
    /// let req = request::Builder::new()
    ///     .method("POST")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn new() -> Builder {
        Builder::default()
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
    /// assert_eq!(req.method_ref(),Some(&Method::GET));
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
    where T : Serialize {
        self.inner.map(move |head| {
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
        F: FnOnce(Parts) -> WebResult<Parts>
    {
        Builder {
            inner: self.inner.and_then(func),
        }
    }
}

impl Default for Builder {
    #[inline]
    fn default() -> Builder {
        Builder {
            inner: Ok(Parts::new()),
        }
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
        Builder { inner: Ok(Parts::new()) }
    }
}

impl<T> Request<T>
    where T : Serialize {
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
        self.parts.url = match self.parts.method {
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
        };
        Ok(())
    }

    pub fn parse(&mut self, buf:&[u8]) -> WebResult<()> {
        self.partial = true;
        let mut buffer = Buffer::new_buf(buf);
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

    pub fn get_header_buffer(&self) -> WebResult<Buffer> {
        

        Err(WebError::Partial)
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
            Some(format!("{}:{}", self.url.domain.as_ref().unwrap(), self.url.port.as_ref().unwrap()))
        } else {
            None
        }
    }
}

impl Default for Request<()> {
    fn default() -> Self {
        Self { parts: Default::default(), body: Default::default(), partial: Default::default() }
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
    where T : Serialize {
    fn serialize(&self, buffer: &mut Buffer) -> WebResult<()> {
        self.parts.method.serialize(buffer)?;
        self.parts.path.serialize(buffer)?;
        buffer.write_u8(b' ').map_err(WebError::from)?;
        self.parts.version.serialize(buffer)?;
        self.parts.header.serialize(buffer)?;
        self.body.serialize(buffer)?;
        Ok(())
    }
}