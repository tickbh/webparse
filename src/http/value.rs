use std::{fmt, io::Write, borrow::Cow};

use crate::{WebError, Helper, Serialize, Buffer, WebResult};

#[derive(Hash, Clone)]
pub enum HeaderValue {
    Stand(&'static str),
    Value(Vec<u8>),
}

impl HeaderValue {
    pub fn from_static(s: &'static str) -> HeaderValue {
        HeaderValue::Stand(s)
    }

    pub fn from_bytes(b: &[u8]) -> HeaderValue {
        HeaderValue::Value(b.to_vec())
    }

    pub fn bytes_len(&self) -> usize {
        match self {
            Self::Stand(s) => s.as_bytes().len(),
            Self::Value(s) => s.len(),
        }
    }
}

impl fmt::Debug for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("HeaderValue");
        match &self {
            Self::Stand(value) => f.field("value", value),
            Self::Value(value) => f.field("value", &String::from_utf8_lossy(value)),
        };
        f.finish()
    }
}


impl TryInto<usize> for &HeaderValue {
    type Error = WebError;

    fn try_into(self) -> Result<usize, WebError> {
        match self {
            HeaderValue::Stand(s) => s.parse().map_err(WebError::from),
            HeaderValue::Value(v) => {
                let mut result = 0usize;
                for b in v {
                    if !Helper::is_digit(*b) {
                        return Err(WebError::IntoError)
                    }
                    match result.overflowing_mul(10) {
                        (u, false) => {
                            result = u + (b - Helper::DIGIT_0) as usize;
                        }
                        (_u, true) => {
                            return Err(WebError::IntoError)
                        }
                    }
                }
                Ok(result)
            }
        }
    }

}


impl TryInto<String> for &HeaderValue {
    type Error = WebError;

    fn try_into(self) -> Result<String, WebError> {
        match self {
            HeaderValue::Stand(s) => Ok(s.to_string()),
            HeaderValue::Value(v) => {
                Ok(String::from_utf8_lossy(v).to_string())
            }
        }
    }
}

impl TryFrom<&'static str> for HeaderValue {
    type Error=WebError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Stand(value))
    }
}

impl TryFrom<String> for HeaderValue {
    type Error=WebError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Value(value.into_bytes()))
    }
}

impl PartialEq<str> for HeaderValue {
    fn eq(&self, other: &str) -> bool {
        match self {
            HeaderValue::Stand(s) => s == &other,
            HeaderValue::Value(s) => &s[..] == other.as_bytes(),
        }
    }
}

impl PartialEq<HeaderValue> for str {
    fn eq(&self, url: &HeaderValue) -> bool {
        url == self
    }
}

impl Serialize for HeaderValue {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        match self {
            Self::Stand(name) => {
                Ok(Cow::Borrowed( name.as_bytes()))
            }
            Self::Value(vec) => {
                Ok(Cow::Borrowed(&**vec))
            }
        }
    }
}