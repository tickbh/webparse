use crate::{WebResult, Url, Scheme, WebError};



pub struct Builder {
    inner: WebResult<Url>
}

impl Builder {

    pub fn new() -> Builder {
        Self::default()
    }
    /// Set the `Scheme` for this URL.
    ///
    /// # Examples
    ///
    /// ```
    /// # use webparse::*;
    ///
    /// let mut builder = Builder::new();
    /// builder.scheme("https");
    /// ```
    pub fn scheme<T>(self, scheme: T) -> Self
    where
        Scheme: TryFrom<T>,
        <Scheme as TryFrom<T>>::Error: Into<crate::WebError>,
    {
        self.map(move |mut inner| {
            let scheme = scheme.try_into().map_err(Into::into)?;
            inner.scheme = scheme;
            Ok(inner)
        })
    }
    
    pub fn username(self, username: String) -> Self
    {
        self.map(move |mut inner| {
            inner.username = Some(username);
            Ok(inner)
        })
    }
    
    pub fn password(self, password: String) -> Self
    {
        self.map(move |mut inner| {
            inner.password = Some(password);
            Ok(inner)
        })
    }
    
    pub fn domain(self, domain: String) -> Self
    {
        self.map(move |mut inner| {
            inner.domain = Some(domain);
            Ok(inner)
        })
    }
    
    pub fn port(self, port: u16) -> Self
    {
        self.map(move |mut inner| {
            inner.port = Some(port);
            Ok(inner)
        })
    }

    pub fn path(self, path: String) -> Self
    {
        self.map(move |mut inner| {
            inner.path = path;
            Ok(inner)
        })
    }
    
    pub fn query(self, query: String) -> Self
    {
        self.map(move |mut inner| {
            inner.query = Some(query);
            Ok(inner)
        })
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(Url) -> Result<Url, WebError>,
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
            inner: Ok(Url::default()),
        }
    }
}
