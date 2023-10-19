use std::fmt::Display;

use crate::{byte_map, Buf, BufMut, Helper, HttpError, Serialize, WebError, WebResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scheme {
    None,
    Http,
    Https,
    Ws,
    Wss,
    Ftp,
    Extension(String),
}

impl Scheme {
    const MAX_SCHEME_LEN: usize = 64;
    // ASCII codes to accept URI string.
    // i.e. A-Z a-z 0-9 !#$%&'*+-._();:@=,/?[]~^
    // TODO: Make a stricter checking for URI string?
    const SCHEME_MAP: [bool; 256] = byte_map![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //  \0                            \n
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //  commands
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0,
        //  \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
        //  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        //  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        //  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        //  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        //  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  del
        //   ====== Extended ASCII (aka. obs-text) ======
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0,
    ];

    #[inline]
    pub(crate) fn is_scheme_token(b: u8) -> bool {
        Self::SCHEME_MAP[b as usize]
    }

    pub fn parse_scheme<T: Buf>(buffer: &mut T) -> WebResult<Scheme> {
        let scheme = Helper::parse_scheme(buffer)?;
        if scheme.as_bytes().len() > Self::MAX_SCHEME_LEN {
            return Err(WebError::Http(HttpError::SchemeTooLong));
        }
        Scheme::try_from(scheme)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::Ws => "ws",
            Scheme::Wss => "wss",
            Scheme::Ftp => "ftp",
            Scheme::Extension(s) => &s.as_str(),
            Scheme::None => "",
        }
    }

    pub fn is_http(&self) -> bool {
        match self {
            Scheme::Http => true,
            _ => false,
        }
    }

    pub fn is_https(&self) -> bool {
        match self {
            Scheme::Https => true,
            _ => false,
        }
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl Serialize for Scheme {
    fn serialize<B: Buf + BufMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        match self {
            Scheme::None => Err(WebError::Serialize("scheme")),
            _ => Ok(buffer.put_slice(self.as_str().as_bytes())),
        }
    }
}

impl TryFrom<&str> for Scheme {
    type Error = WebError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() > 64 {
            return Err(WebError::from(crate::UrlError::UrlInvalid));
        }
        match value {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            "ws" => Ok(Scheme::Ws),
            "wss" => Ok(Scheme::Wss),
            "ftp" => Ok(Scheme::Ftp),
            _ => Ok(Scheme::Extension(value.to_string())),
        }
    }
}
