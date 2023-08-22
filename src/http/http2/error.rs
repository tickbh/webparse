use std::fmt;
use crate::WebError;

use super::{DecoderError, HuffmanDecoderError};


#[derive(Debug)]
pub enum Http2Error {
    Decoder(DecoderError),
    Huffman(HuffmanDecoderError),

}


impl Http2Error {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
            Self::Decoder(_) => "",
            Self::Huffman(_) => "",
        }
    }

    pub fn into<E: Into<Http2Error>>(e: E) -> WebError {
        WebError::Http2(e.into())
    }
}

impl fmt::Display for Http2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description_str())
    }
}

impl From<DecoderError> for Http2Error {
    fn from(e: DecoderError) -> Self {
        Http2Error::Decoder(e)
    }
}

impl From<HuffmanDecoderError> for Http2Error {
    fn from(e: HuffmanDecoderError) -> Self {
        Http2Error::Huffman(e)
    }
}
