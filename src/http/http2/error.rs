use std::fmt;
use crate::WebError;

use super::{DecoderError, HuffmanDecoderError};


#[derive(Debug)]
pub enum Http2Error {
    Decoder(DecoderError),
    Huffman(HuffmanDecoderError),
    /// A full frame header was not passed.
    Short,

    /// An unsupported value was set for the flag value.
    BadFlag(u8),

    /// An unsupported value was set for the frame kind.
    BadKind(u8),

    /// The padding length was larger than the frame-header-specified
    /// length of the payload.
    TooMuchPadding(u8),

    /// The payload length specified by the frame header was shorter than
    /// necessary for the parser settings specified and the frame type.
    ///
    /// This happens if, for instance, the priority flag is set and the
    /// header length is shorter than a stream dependency.
    ///
    /// `PayloadLengthTooShort` should be treated as a protocol error.
    PayloadLengthTooShort,

    /// The payload length specified by the frame header of a settings frame
    /// was not a round multiple of the size of a single setting.
    PartialSettingLength,

    /// The payload length specified by the frame header was not the
    /// value necessary for the specific frame type.
    InvalidPayloadLength,
    /// 无效的streamId, 比如setting只能以0的id来传送
    InvalidStreamId,
    /// 无效的设置值, 比如enable_push只能取0和1
    InvalidSettingValue,
    /// 无效的frame大小 
    BadFrameSize,
    /// 无效的窗口大小文件
    InvalidWindowUpdateValue,
    /// 无效的依赖StreamId
    InvalidDependencyId,
}


impl Http2Error {
    #[inline]
    pub fn description_str(&self) -> &'static str {
        match *self {
            Self::Decoder(_) => "",
            Self::Huffman(_) => "",
            _ => "",
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

impl Into<WebError> for Http2Error {
    fn into(self) -> WebError {
        WebError::Http2(self)
    }
}