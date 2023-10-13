use std::hash::Hash;
use std::{borrow::Cow, fmt};

use crate::{Helper, WebError, WebResult, Buf, BufMut};

#[derive(Clone, Debug)]
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

    pub fn from_cow(b: Cow<[u8]>) -> HeaderValue {
        HeaderValue::Value(Vec::from(b.to_owned()))
    }

    pub fn bytes_len(&self) -> usize {
        match self {
            Self::Stand(s) => s.as_bytes().len(),
            Self::Value(s) => s.len(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Stand(s) => &s.as_bytes(),
            Self::Value(s) => &s,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::Stand(s) => Some(s.to_string()),
            Self::Value(s) => String::from_utf8(s.clone()).map_or(None, |s| Some(s)),
        }
    }

    pub fn push(&mut self, value: HeaderValue) {
        match self {
            Self::Stand(s) => *self = Self::Value(s.as_bytes().to_vec()),
            _ => {}
        }
        match self {
            Self::Value(s) => {
                s.push(b';');
                s.append(&mut value.as_bytes().to_vec());
            },
            _ => unreachable!()
        }
    }

    pub fn encode<B: Buf+BufMut>(&self, buffer: &mut B) -> WebResult<usize> {
        match self {
            Self::Stand(name) => Ok(buffer.put_slice(name.as_bytes())),
            Self::Value(vec) => Ok(buffer.put_slice(&**vec)),
        }
    }

    pub fn contains(&self, bytes: &[u8]) -> bool {
        Helper::contains_bytes(self.as_bytes(), bytes)
    }
}

impl Hash for HeaderValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            HeaderValue::Stand(stand) => {
                (*stand.as_bytes()).hash(state);
            }
            HeaderValue::Value(val) => {
                val.hash(state);
            }
        }
    }
}

impl fmt::Display for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Stand(value) => f.write_str(value),
            Self::Value(value) => f.write_str(&String::from_utf8_lossy(value)),
        }
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
                        return Err(WebError::IntoError);
                    }
                    match result.overflowing_mul(10) {
                        (u, false) => {
                            result = u + (b - Helper::DIGIT_0) as usize;
                        }
                        (_u, true) => return Err(WebError::IntoError),
                    }
                }
                Ok(result)
            }
        }
    }
}


impl TryInto<isize> for &HeaderValue {
    type Error = WebError;

    fn try_into(self) -> Result<isize, WebError> {
        match self {
            HeaderValue::Stand(s) => s.parse().map_err(WebError::from),
            HeaderValue::Value(v) => {
                let mut result = 0isize;
                let mut is_neg = false;
                for b in v {
                    if !Helper::is_digit(*b) {
                        if b == &b'-' && result == 0 {
                            is_neg = true;
                            continue;
                        }
                        return Err(WebError::IntoError);
                    }
                    match result.overflowing_mul(10) {
                        (u, false) => {
                            result = u + (b - Helper::DIGIT_0) as isize;
                        }
                        (_u, true) => return Err(WebError::IntoError),
                    }
                }
                if is_neg {
                    Ok(-result)
                } else {
                    Ok(result)
                }
            }
        }
    }
}

impl TryInto<String> for &HeaderValue {
    type Error = WebError;

    fn try_into(self) -> Result<String, WebError> {
        match self {
            HeaderValue::Stand(s) => Ok(s.to_string()),
            HeaderValue::Value(v) => Ok(String::from_utf8_lossy(v).to_string()),
        }
    }
}

impl TryFrom<&'static str> for HeaderValue {
    type Error = WebError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Stand(value))
    }
}

impl TryFrom<String> for HeaderValue {
    type Error = WebError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Value(value.into_bytes()))
    }
}

impl TryFrom<usize> for HeaderValue {
    type Error = WebError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Value(format!("{}", value).into_bytes()))
    }
}

impl TryFrom<isize> for HeaderValue {
    type Error = WebError;
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Ok(HeaderValue::Value(format!("{}", value).into_bytes()))
    }
}


impl Eq for HeaderValue {}

impl PartialEq<HeaderValue> for HeaderValue {
    fn eq(&self, other: &HeaderValue) -> bool {
        match (self, other) {
            (Self::Stand(l0), Self::Stand(r0)) => l0 == r0,
            (Self::Value(l0), Self::Value(r0)) => l0 == r0,
            (Self::Stand(l0), Self::Value(r0)) => l0.as_bytes() == r0,
            (Self::Value(l0), Self::Stand(r0)) => l0 == r0.as_bytes(),
        }
    }
}

impl PartialEq<&str> for HeaderValue {
    fn eq(&self, other: &&str) -> bool {
        match self {
            HeaderValue::Stand(s) => s == other,
            HeaderValue::Value(s) => &s[..] == other.as_bytes(),
        }
    }
}

impl PartialEq<HeaderValue> for [u8] {
    fn eq(&self, other: &HeaderValue) -> bool {
        other == self
    }
}

impl PartialEq<[u8]> for HeaderValue {
    fn eq(&self, other: &[u8]) -> bool {
        match self {
            HeaderValue::Stand(s) => s.as_bytes() == other,
            HeaderValue::Value(s) => &s[..] == other,
        }
    }
}

impl PartialEq<HeaderValue> for &str {
    fn eq(&self, url: &HeaderValue) -> bool {
        url == self
    }
}

