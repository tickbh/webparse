use crate::{Buffer, WebResult, WebError};

use super::{Method, Version, HeaderMap, HeaderName, HeaderValue};



pub struct Helper;

impl Helper {
    
    /// Determines if byte is a token char.
    ///
    /// > ```notrust
    /// > token          = 1*tchar
    /// >
    /// > tchar          = "!" / "#" / "$" / "%" / "&" / "'" / "*"
    /// >                / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
    /// >                / DIGIT / ALPHA
    /// >                ; any VCHAR, except delimiters
    /// > ```
    #[inline]
    fn is_token(b: u8) -> bool {
        b > 0x1F && b < 0x7F
    }

    // ASCII codes to accept URI string.
    // i.e. A-Z a-z 0-9 !#$%&'*+-._();:@=,/?[]~^
    // TODO: Make a stricter checking for URI string?
    const URI_MAP: [bool; 256] = byte_map![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //  \0                            \n
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //  commands
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1,
    //  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    //  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  del
    //   ====== Extended ASCII (aka. obs-text) ======
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[inline]
    pub(crate) fn is_uri_token(b: u8) -> bool {
        Self::URI_MAP[b as usize]
    }

    const HEADER_NAME_MAP: [bool; 256] = byte_map![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[inline]
    pub(crate) fn is_header_name_token(b: u8) -> bool {
        Self::HEADER_NAME_MAP[b as usize]
    }

    const HEADER_VALUE_MAP: [bool; 256] = byte_map![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ];


    #[inline]
    pub(crate) fn is_header_value_token(b: u8) -> bool {
        Self::HEADER_VALUE_MAP[b as usize]
    }

    pub(crate) fn parse_method(buffer: &mut Buffer) -> WebResult<Method> {
        let token = Self::parse_token(buffer)?;
        match token {
            Method::SGET => Ok(Method::GET),
            Method::SPOST => Ok(Method::POST),
            Method::SPUT => Ok(Method::PUT),
            Method::SDELETE => Ok(Method::DELETE),
            Method::SHEAD => Ok(Method::HEAD),
            Method::SOPTIONS => Ok(Method::OPTIONS),
            Method::SCONNECT => Ok(Method::CONNECT),
            Method::SPATCH => Ok(Method::PATCH),
            Method::STRACE => Ok(Method::TRACE),
            _ => {
                Ok(Method::Extension(token.to_string()))
            }
        }
    }


    pub(crate) fn parse_version(buffer: &mut Buffer) -> WebResult<Version> {
        let token = Self::parse_token(buffer)?;
        match token {
            Version::SHTTP10 => Ok(Version::Http10),
            Version::SHTTP11 => Ok(Version::Http11),
            Version::SHTTP2 => Ok(Version::Http2),
            Version::SHTTP3 => Ok(Version::Http3),
            _ => {
                Err(WebError::Version)
            }
        }
    }

    #[inline]
    pub(crate) fn parse_token_by_func<'a>(buffer: &'a mut Buffer, func: fn(u8)->bool, err: WebError) -> WebResult<&'a str> {
        let mut b = next!(buffer)?;
        if !func(b) {
            return Err(err);
        }

        loop {
            b = next!(buffer)?;
            if b == b' ' {
                return Ok(
                    unsafe {
                        std::str::from_utf8_unchecked(buffer.slice_skip(1))
                    })
            } else if !func(b) {
                buffer.retreat(1);
                return Ok(
                    unsafe {
                        std::str::from_utf8_unchecked(buffer.slice())
                    })
            }
        }
    }


    #[inline]
    pub(crate) fn parse_token<'a>(buffer: &'a mut Buffer) -> WebResult<&'a str> {
        Self::parse_token_by_func(buffer, Self::is_token, WebError::Token)
    }

    #[inline]
    pub(crate) fn parse_header_name<'a>(buffer: &'a mut Buffer) -> WebResult<HeaderName> {
        let token = Self::parse_token_by_func(buffer, Self::is_header_name_token, WebError::HeaderName)?;
        match HeaderName::from_bytes(token.as_bytes()) {
            Some(name) => Ok(name),
            _ => Err(WebError::HeaderName)
        }
    }

    #[inline]
    pub(crate) fn parse_header_value<'a>(buffer: &'a mut Buffer) -> WebResult<HeaderValue> {
        let token = Self::parse_token_by_func(buffer, Self::is_header_value_token, WebError::HeaderValue)?;
        Ok(HeaderValue::Value(token.as_bytes().to_vec()))
    }

    #[inline]
    pub(crate) fn skip_new_line(buffer: &mut Buffer) -> WebResult<()> {
        match next!(buffer)? {
            b'\r' => {
                expect!(buffer.next() == b'\n' => Err(WebError::NewLine));
                buffer.slice();
            },
            b'\n' => {
                buffer.slice();
            },
            b' ' => {
            },
            _ => return Err(WebError::NewLine)
        };
        Ok(())
    }

    #[inline]
    pub(crate) fn skip_empty_lines(buffer: &mut Buffer) -> WebResult<()> {
        loop {
            let b = buffer.peek();
            match b {
                Some(b'\r') => {
                    buffer.bump();
                    expect!(buffer.next() == b'\n' => Err(WebError::NewLine));
                }
                Some(b'\n') => {
                    buffer.bump();
                }
                Some(..) => {
                    buffer.slice();
                    return Ok(());
                }
                None => return Err(WebError::Partial),
            }
        }
    }

    #[inline]
    pub(crate) fn skip_spaces(buffer: &mut Buffer) -> WebResult<()> {
        loop {
            let b = buffer.peek();
            match b {
                Some(b' ') => {
                    buffer.bump();
                }
                Some(..) => {
                    buffer.slice();
                    return Ok(());
                }
                None => return Err(WebError::Partial),
            }
        }
    }
    
    #[inline]
    pub(crate) fn parse_header(buffer: &mut Buffer) -> WebResult<HeaderMap> {
        let mut header = HeaderMap::new();
        
        loop {
            let b = peek!(buffer)?;
            if b == b'\r' {
                buffer.next();
                expect!(buffer.next() == b'\n' => Err(WebError::NewLine));
                return Ok(header);
            }
            if b == b'\n' {
                buffer.next();
                return Ok(header);
            }

            let name = Helper::parse_header_name(buffer)?;
            Self::skip_spaces(buffer)?;
            expect!(buffer.next() == b':' => Err(WebError::HeaderName));
            Self::skip_spaces(buffer)?;
            let value = Helper::parse_header_value(buffer)?;
            header.headers.insert(name, value);
            Self::skip_new_line(buffer)?;
        }
    }

}
