use std::{
    any::{Any},
    sync::{Arc, RwLock}, fmt::Display,
};

use crate::{
    Binary, BinaryMut, Buf, BufMut, Extensions, HeaderMap, HeaderName, HeaderValue, Serialize, Version, WebError, WebResult, Helper,
};

use super::{
    http2::{HeaderIndex},
    StatusCode,
};

#[derive(Debug)]
pub struct Response<T>
where
    T: Serialize,
{
    parts: Parts,
    body: T,
    partial: bool,
}

#[derive(Debug)]
pub struct Parts {
    pub status: StatusCode,
    pub header: HeaderMap,
    pub version: Version,
    pub extensions: Extensions,
}

#[derive(Debug)]
pub struct Builder {
    inner: WebResult<Parts>,
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct either a
    /// `Head` or a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    ///
    /// let response = response::Builder::new()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn new() -> Builder {
        Builder::default()
    }

    /// Set the HTTP status for this response.
    ///
    /// By default this is `200`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    ///
    /// let response = Response::builder()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn status<T>(self, status: T) -> Builder
    where
        StatusCode: TryFrom<T>,
        <StatusCode as TryFrom<T>>::Error: Into<WebError>,
    {
        self.and_then(move |mut head| {
            head.status = TryFrom::try_from(status).map_err(Into::into)?;
            Ok(head)
        })
    }

    /// Set the HTTP version for this response.
    ///
    /// By default this is HTTP/1.1
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    ///
    /// let response = Response::builder()
    ///     .version(Version::HTTP_2)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn version(self, version: Version) -> Builder {
        self.and_then(move |mut head| {
            head.version = version;
            Ok(head)
        })
    }

    /// Appends a header to this response builder.
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
    /// let response = Response::builder()
    ///     .header("Content-Type", "text/html")
    ///     .header("X-Custom-Foo", "bar")
    ///     .header("content-length", 0)
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

    /// Get header on this response builder.
    ///
    /// When builder has error returns None.
    ///
    /// # Example
    ///
    /// ```
    /// # use webparse::Response;
    /// # use webparse::HeaderValue;
    /// let res = Response::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( &headers["Accept"], "text/html" );
    /// assert_eq!( &headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_ref(&self) -> Option<&HeaderMap> {
        self.inner.as_ref().ok().map(|h| &h.header)
    }

    /// Get header on this response builder.
    /// when builder has error returns None
    ///
    /// # Example
    ///
    /// ```
    /// # use webparse::*;
    /// # use webparse::header::HeaderValue;
    /// # use webparse::response::Builder;
    /// let mut res = Response::builder();
    /// {
    ///   let headers = res.headers_mut().unwrap();
    ///   headers.insert("Accept", HeaderValue::from_static("text/html"));
    ///   headers.insert("X-Custom-Foo", HeaderValue::from_static("bar"));
    /// }
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap> {
        self.inner.as_mut().ok().map(|h| &mut h.header)
    }

    /// Adds an extension to this builder
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    ///
    /// let response = Response::builder()
    ///     .extension("My Extension")
    ///     .body(())
    ///     .unwrap();
    ///
    /// assert_eq!(response.extensions().get::<&'static str>(),
    ///            Some(&"My Extension"));
    /// ```
    pub fn extension<T>(self, extension: T) -> Builder
    where
        T: Any + Send + Sync + 'static,
    {
        self.and_then(move |mut head| {
            head.extensions.insert(extension);
            Ok(head)
        })
    }

    /// Get a reference to the extensions for this response builder.
    ///
    /// If the builder has an error, this returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use webparse::Response;
    /// let res = Response::builder().extension("My Extension").extension(5u32);
    /// let extensions = res.extensions_ref().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|h| &h.extensions)
    }

    // /// Get a mutable reference to the extensions for this response builder.
    // ///
    // /// If the builder has an error, this returns `None`.
    // ///
    // /// # Example
    // ///
    // /// ```
    // /// # use webparse::Response;
    // /// let mut res = Response::builder().extension("My Extension");
    // /// let mut extensions = res.extensions_mut().unwrap();
    // /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    // /// extensions.insert(5u32);
    // /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    // /// ```
    // pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
    //     self.inner.as_mut().ok().map(|h| &mut h.extensions)
    // }

    /// "Consumes" this builder, using the provided `body` to return a
    /// constructed `Response`.
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
    /// let response = Response::builder()
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn body<T: Serialize>(self, body: T) -> WebResult<Response<T>> {
        self.inner.map(move |parts: Parts| Response {
            parts,
            body,
            partial: false,
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
        Builder {
            inner: Ok(Parts::default()),
        }
    }
}

impl Response<()> {
    /// Creates a new builder-style object to manufacture a `Response`
    ///
    /// This method returns an instance of `Builder` which can be used to
    /// create a `Response`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response = Response::builder()
    ///     .status(200)
    ///     .header("X-Custom-Foo", "Bar")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl<T: Serialize> Response<T> {
    /// Creates a new blank `Response` with the body
    ///
    /// The component ports of this response will be set to their default, e.g.
    /// the ok status, no headers, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response = Response::new("hello world");
    ///
    /// assert_eq!(response.status(), StatusCode::OK);
    /// assert_eq!(*response.body(), "hello world");
    /// ```
    #[inline]
    pub fn new(body: T) -> Response<T> {
        Response {
            parts: Parts::default(),
            body: body,
            partial: false,
        }
    }

    /// Creates a new `Response` with the given head and body
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response = Response::new("hello world");
    /// let (mut parts, body) = response.into_parts();
    ///
    /// parts.status = StatusCode::BAD_REQUEST;
    /// let response = Response::from_parts(parts, body);
    ///
    /// assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    /// assert_eq!(*response.body(), "hello world");
    /// ```
    #[inline]
    pub fn from_parts(parts: Parts, body: T) -> Response<T> {
        Response {
            parts: parts,
            body: body,
            partial: false,
        }
    }

    pub fn is_partial(&self) -> bool {
        self.partial
    }

    /// Returns the `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<()> = Response::default();
    /// assert_eq!(response.status(), StatusCode::OK);
    /// ```
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.parts.status
    }

    /// Returns a mutable reference to the associated `StatusCode`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let mut response: Response<()> = Response::default();
    /// *response.status_mut() = StatusCode::CREATED;
    /// assert_eq!(response.status(), StatusCode::CREATED);
    /// ```
    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.parts.status
    }

    /// Returns a reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<()> = Response::default();
    /// assert_eq!(response.version(), Version::HTTP_11);
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.parts.version
    }

    /// Returns a mutable reference to the associated version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let mut response: Response<()> = Response::default();
    /// *response.version_mut() = Version::HTTP_2;
    /// assert_eq!(response.version(), Version::HTTP_2);
    /// ```
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.parts.version
    }

    /// Returns a reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<()> = Response::default();
    /// assert!(response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.parts.header
    }

    /// Returns a mutable reference to the associated header field map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// # use webparse::header::*;
    /// let mut response: Response<()> = Response::default();
    /// response.headers_mut().insert(HOST, HeaderValue::from_static("world"));
    /// assert!(!response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.parts.header
    }

    /// Returns a reference to the associated extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<()> = Response::default();
    /// assert!(response.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.parts.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.parts.extensions
    }

    // /// Returns a mutable reference to the associated extensions.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// # use webparse::*;
    // /// # use webparse::header::*;
    // /// let mut response: Response<()> = Response::default();
    // /// response.extensions_mut().insert("hello");
    // /// assert_eq!(response.extensions().get(), Some(&"hello"));
    // /// ```
    // #[inline]
    // pub fn extensions_mut(&mut self) -> &mut Extensions {
    //     &mut self.parts.extensions
    // }

    /// Returns a reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<String> = Response::default();
    /// assert!(response.body().is_empty());
    /// ```
    #[inline]
    pub fn body(&self) -> &T {
        &self.body
    }

    /// Returns a mutable reference to the associated HTTP body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let mut response: Response<String> = Response::default();
    /// response.body_mut().push_str("hello world");
    /// assert!(!response.body().is_empty());
    /// ```
    #[inline]
    pub fn body_mut(&mut self) -> &mut T {
        &mut self.body
    }

    /// Consumes the response, returning just the body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::Response;
    /// let response = Response::new(10);
    /// let body = response.into_body();
    /// assert_eq!(body, 10);
    /// ```
    #[inline]
    pub fn into_body(self) -> T {
        self.body
    }

    /// Consumes the response returning the head and body parts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response: Response<()> = Response::default();
    /// let (parts, body) = response.into_parts();
    /// assert_eq!(parts.status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn into_parts(self) -> (Parts, T) {
        (self.parts, self.body)
    }

    /// Consumes the response returning a new response with body mapped to the
    /// return type of the passed in function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    /// let response = Response::builder().body("some string").unwrap();
    /// let mapped_response: Response<&[u8]> = response.map(|b| {
    ///   assert_eq!(b, "some string");
    ///   b.as_bytes()
    /// });
    /// assert_eq!(mapped_response.body(), &"some string".as_bytes());
    /// ```
    #[inline]
    pub fn map<F, U: Serialize>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        Response {
            body: f(self.body),
            parts: self.parts,
            partial: self.partial,
        }
    }

    pub fn httpdata(&mut self) -> WebResult<Vec<u8>> {
        let mut buffer = BinaryMut::new();
        self.serialize(&mut buffer)?;
        return Ok(buffer.into_slice_all());
    }

    pub fn into<B: Serialize>(self, body: B) -> (Response<B>, T) {
        let new = Response {
            body,
            parts: self.parts,
            partial: self.partial,
        };
        (new, self.body)
    }

    pub fn into_type<B: From<T> + Serialize>(self) -> Response<B> {
        let new = Response {
            body: From::from(self.body),
            parts: self.parts,
            partial: self.partial,
        };
        new
    }

    pub fn into_binary(mut self) -> Response<Binary> {
        let mut binary = BinaryMut::new();
        let _ = self.body.serialize(&mut binary);
        let new = Response {
            body: binary.freeze(),
            parts: self.parts,
            partial: self.partial,
        };
        new
    }

    
    /// 获取返回的body长度, 如果为0则表示未写入信息
    pub fn get_body_len(&self) -> usize {
        self.parts.header.get_body_len()
    }

    pub fn encode_header<B: Buf + BufMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += self.parts.version.encode(buffer)?;
        size += buffer.put_slice(" ".as_bytes());
        size += self.parts.status.encode(buffer)?;
        size += self.parts.header.encode(buffer)?;
        Ok(size)
    }


    pub fn parse_buffer<B: Buf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        self.partial = true;
        Helper::skip_empty_lines(buffer)?;
        self.parts.version = Helper::parse_version(buffer)?;
        Helper::skip_spaces(buffer)?;
        self.parts.status = Helper::parse_status(buffer)?;
        Helper::skip_spaces(buffer)?;
        let _reason = Helper::parse_token(buffer)?;
        Helper::skip_new_line(buffer)?;
        Helper::parse_header(buffer, &mut self.parts.header)?;
        self.partial = false;
        Ok(buffer.mark_commit())
    }
}

impl<T: Default + Serialize> Default for Response<T> {
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
        Self {
            status: StatusCode::OK,
            header: HeaderMap::new(),
            version: Version::Http11,
            extensions: Extensions::new(),
        }
    }
}

impl Clone for Parts {
    fn clone(&self) -> Self {
        let mut value = Self {
            status: self.status.clone(),
            header: self.header.clone(),
            version: self.version.clone(),
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

impl<T> Serialize for Response<T>
where
    T: Serialize,
{
    fn serialize<B: Buf + BufMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += self.parts.version.encode(buffer)?;
        size += buffer.put_slice(" ".as_bytes());
        size += self.parts.status.encode(buffer)?;
        size += self.parts.header.encode(buffer)?;
        size += self.body.serialize(buffer)?;
        Ok(size)
    }
}

impl<T> Display for Response<T>
where T: Serialize + Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.parts.version.fmt(f)?;
        f.write_str(" ")?;
        self.parts.status.fmt(f)?;
        f.write_str("\r\n")?;
        self.parts.header.fmt(f)?;
        self.body.fmt(f)
    }
}