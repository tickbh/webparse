use crate::{Buf, WebResult, WebError, byte_map, next, expect, peek, HttpError, http::StatusCode, BufMut};
use super::{Method, Version, HeaderMap, HeaderName, HeaderValue, Scheme};


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
    pub fn is_token(b: u8) -> bool {
        b > 0x1F && b < 0x7F && b != b' '
    }


    #[inline]
    pub fn is_status_token(b: u8) -> bool {
        b > 0x1F && b < 0x7F
    }
    
    #[inline]
    pub fn is_alpha(b: u8) -> bool {
        if b >= 65 && b <= 90 {
            return true
        } else if b >= 97 && b <= 122 {
            return true
        } else {
            return false
        }
    }

    pub const DIGIT_0 :u8 = 48;

    #[inline]
    pub fn is_digit(b: u8) -> bool {
        if b >= 48 && b <= 57 {
            return true
        } else {
            return false
        }
    }

    #[inline]
    pub fn is_hex(b: u8) -> bool {
        if b >= 48 && b <= 57 {
            return true
        } else if b >= 65 && b <= 70 {
            return true
        } else if b >= 97 && b <= 102 {
            return true
        } else {
            return false
        }
    }

    pub fn to_hex(b: u8) -> u8 {
        Self::HEX_MAP[b as usize]
    }

    #[inline]
    pub fn convert_hex(b: u8) -> Option<u8> {
        if b >= 48 && b <= 57 {
            return Some(b - 48)
        } else if b >= 65 && b <= 70 {
            return Some(b - 65 + 10)
        } else if b >= 97 && b <= 102 {
            return Some(b - 97 + 10)
        } else {
            return None;
        }
    }

    const HEX_MAP: [u8; 16] = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', 
                                b'9', b'A', b'B', b'C', b'D', b'E', b'F'];

    // ASCII codes to accept URI string.
    // i.e. A-Z a-z 0-9 !#$%&'*+-._();:@=,/?[]~^
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
    pub fn is_uri_token(b: u8) -> bool {
        Self::URI_MAP[b as usize]
    }
    
    // ASCII codes to accept URI string.
    // i.e. A-Z a-z 0-9 &:?/-._~
    const URITRANS_MAP: [bool; 256] = byte_map![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //  \0                            \n
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    //  commands
        0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1,
    //  \w !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1,
    //  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
    //  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    //  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
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
    pub(crate) fn is_not_uritrans(b: u8) -> bool {
        Self::URITRANS_MAP[b as usize]
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
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
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

    pub(crate) fn parse_method<B:Buf>(buffer: &mut B) -> WebResult<Method> {
        let token = Self::parse_token(buffer)?;
        TryFrom::try_from(token)
    }

    pub(crate) fn parse_status<B:Buf>(buffer: &mut B) -> WebResult<StatusCode> {
        let token = Self::parse_token(buffer)?;
        let status = StatusCode::try_from(token);


        status
    }

    pub(crate) fn parse_version<B:Buf>(buffer: &mut B) -> WebResult<Version> {
        let token = Self::parse_token(buffer)?;
        match token {
            Version::SHTTP10 => Ok(Version::Http10),
            Version::SHTTP11 => Ok(Version::Http11),
            Version::SHTTP2 => Ok(Version::Http2),
            Version::SHTTP3 => Ok(Version::Http3),
            _ => {
                Err(WebError::from(HttpError::Version))
            }
        }
    }

    
    #[inline]
    pub(crate) fn parse_token_by_func_empty<'a, B: Buf>(buffer: &'a mut B, func: fn(u8)->bool, err: WebError, empty: bool) -> WebResult<&'a str> {
        let mut b = next!(buffer)?;
        if !func(b) {
            if empty {
                return Ok("");
            }
            return Err(err);
        }

        loop {
            b = peek!(buffer)?;
            if !func(b) {
                return Ok(
                    unsafe {
                        std::str::from_utf8_unchecked(buffer.mark_slice())
                    })
            }
            next!(buffer)?;
        }
    }

    #[inline]
    pub(crate) fn parse_token_by_func<'a, B: Buf>(buffer: &'a mut B, func: fn(u8)->bool, err: WebError) -> WebResult<&'a str> {
        Self::parse_token_by_func_empty(buffer, func, err, false)
    }

    #[inline]
    pub(crate) fn parse_hex<'a, B: Buf>(buffer: &'a mut B) -> WebResult<&'a str> {
        Self::parse_token_by_func(buffer, Self::is_hex, WebError::from(HttpError::Token))
    }

    #[inline]
    pub(crate) fn parse_token<'a, B:Buf>(buffer: &'a mut B) -> WebResult<&'a str> {
        Self::parse_token_by_func(buffer, Self::is_token, WebError::from(HttpError::Token))
    }

    #[inline]
    pub(crate) fn parse_status_token<'a, B:Buf>(buffer: &'a mut B) -> WebResult<&'a str> {
        Self::parse_token_by_func(buffer, Self::is_status_token, WebError::from(HttpError::Token))
    }

    #[inline]
    pub(crate) fn parse_header_name<'a, B:Buf>(buffer: &'a mut B) -> WebResult<HeaderName> {
        let token = Self::parse_token_by_func(buffer, Self::is_header_name_token, WebError::from(HttpError::HeaderName))?;
        match HeaderName::from_bytes(token.as_bytes()) {
            Some(name) => Ok(name),
            _ => Err(WebError::from(HttpError::from(HttpError::HeaderName)))
        }
    }

    #[inline]
    pub(crate) fn parse_header_value<'a, B:Buf>(buffer: &'a mut B) -> WebResult<HeaderValue> {
        let token = Self::parse_token_by_func_empty(buffer, Self::is_header_value_token, WebError::from(HttpError::HeaderValue), true)?;
        Ok(HeaderValue::Value(token.as_bytes().to_vec()))
    }

    #[inline]
    pub(crate) fn parse_scheme<'a, B:Buf>(buffer: &'a mut B) -> WebResult<&'a str> {
        let token = Self::parse_token_by_func(buffer, Scheme::is_scheme_token, WebError::from(HttpError::HeaderValue))?;
        Ok(token)
    }

    #[inline]
    pub(crate) fn skip_new_line<B:Buf>(buffer: &mut B) -> WebResult<()> {
        match next!(buffer)? {
            b'\r' => {
                expect!(buffer.next() == b'\n' => Err(WebError::from(HttpError::NewLine)));
                buffer.mark_slice();
            },
            b'\n' => {
                buffer.mark_slice();
            },
            b' ' => {
            },
            _ => return Err(WebError::from(HttpError::Partial))
        };
        Ok(())
    }

    #[inline]
    pub(crate) fn skip_empty_lines<B: Buf>(buffer: &mut B) -> WebResult<()> {
        loop {
            let b = buffer.peek();
            match b {
                Some(b'\r') => {
                    buffer.mark_bump();
                    expect!(buffer.next() == b'\n' => Err(WebError::from(HttpError::NewLine)));
                }
                Some(b'\n') => {
                    buffer.mark_bump();
                }
                Some(..) => {
                    buffer.mark_slice();
                    return Ok(());
                }
                None => return Err(WebError::from(HttpError::Partial)),
            }
        }
    }

    #[inline]
    pub(crate) fn skip_spaces<B:Buf>(buffer: &mut B) -> WebResult<()> {
        loop {
            let b = buffer.peek();
            match b {
                Some(b' ') => {
                    buffer.mark_bump();
                }
                Some(..) => {
                    buffer.mark_slice();
                    return Ok(());
                }
                None => return Err(WebError::from(HttpError::Partial)),
            }
        }
    }
    
    #[inline]
    pub(crate) fn parse_header<B:Buf>(buffer: &mut B, header: &mut HeaderMap) -> WebResult<()> {
        header.clear();

        loop {
            let b = peek!(buffer)?;
            if b == b'\r' {
                buffer.get_next();
                expect!(buffer.next() == b'\n' => Err(WebError::from(HttpError::NewLine)));
                return Ok(());
            }
            if b == b'\n' {
                buffer.get_next();
                return Ok(());
            }

            let name = Helper::parse_header_name(buffer)?;
            Self::skip_spaces(buffer)?;
            expect!(buffer.next() == b':' => Err(WebError::from(HttpError::HeaderName)));
            Self::skip_spaces(buffer)?;
            let value = Helper::parse_header_value(buffer)?;
            Self::skip_new_line(buffer)?;
            header.insert(name, value);
        }
    }

    pub fn parse_chunk_data<B:Buf>(buffer: &mut B) -> WebResult<(Vec<u8>, usize, bool)> {
        let first = buffer.mark_commit();
        let num = Helper::parse_hex(buffer)?;
        
        let num = usize::from_str_radix(num, 16).unwrap();
        if num == 0 {
            println!("receiver end message");
        }
        Helper::skip_new_line(buffer)?;
        if num + 2 > buffer.remaining() {
            return Err(WebError::Http(HttpError::Partial));
        }

        let ret = buffer.chunk()[..num].to_vec();
        buffer.advance(num);
        Helper::skip_new_line(buffer)?;
        println!("chunks = {}, is_end = {}", buffer.mark_commit() - first, num == 0);
        Ok((ret, buffer.mark_commit() - first, num == 0))
    }

    pub fn encode_chunk_data<B:Buf+BufMut>(buffer: &mut B, data: &[u8]) -> std::io::Result<usize> {
        let len_str = format!("{:x}", data.len());
        println!("write chunk len = {}", len_str);
        let mut size = buffer.put_slice(len_str.as_bytes());
        size += buffer.put_slice("\r\n".as_bytes());
        size += buffer.put_slice(data);
        size += buffer.put_slice("\r\n".as_bytes());
        Ok(size)
    }

    #[inline]
    pub fn hex_to_vec(s: &str) -> Vec<u8> {
        let mut result = vec![];
        let bytes = s.as_bytes();
        let mut val = 0;
        let mut is_first = true;
        for b in bytes {
            if b != &b' ' {
                if is_first {
                    val = u8::from_str_radix(std::str::from_utf8(&[*b]).unwrap(), 16).unwrap();
                    is_first = false
                } else {
                    val = val * 16 + u8::from_str_radix(std::str::from_utf8(&[*b]).unwrap(), 16).unwrap();
                    result.push(val);
                    val = 0;
                    is_first = true;
                }
            }
        }
        result
    }


    pub fn eq_bytes_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        for i in 0..a.len() {
            if a[i] == b[i] {
                continue;
            }
            let wrap = a[i].wrapping_sub(b[i]);
            if wrap != 32 && wrap != 224 {
                return false;
            }
        }
        true
    }
    
    pub fn eq_bytes(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a == b
    }
    
    pub fn contains_bytes(a: &[u8], b: &[u8]) -> bool {
        if a.len() < b.len() {
            return false;
        }
        for i in 0..(a.len() - b.len() + 1) {
            if &a[i..(i + b.len())] == b {
                return true;
            }
        }
        false
    }
}
