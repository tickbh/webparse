use crate::{WebError, Helper};




#[derive(Debug, Hash)]
pub enum HeaderValue {
    Stand(&'static str),
    Value(Vec<u8>),
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
