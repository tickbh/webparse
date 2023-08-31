pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
use std::{fmt::Debug, borrow::Cow};
mod error;
mod flag;
mod frame;
mod hpack;
mod kind;
mod payload;

pub use error::Http2Error;
pub use flag::Flag;
pub use frame::Frame;
pub use kind::Kind;
pub use payload::Payload;

use crate::{serialize, BinaryMut, Buf, BufMut, MarkBuf, Request, WebResult, Method, Response, WebError, Serialize};
pub use hpack::*;

use self::frame::FrameHeader;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse<T: Buf + MarkBuf>(buf: &mut T) -> StreamIdentifier {
        if buf.remaining() < 4 {
            return StreamIdentifier(0);
        }
        StreamIdentifier(read_u31(buf))
    }
}

impl Serialize for StreamIdentifier {
    fn serial_bytes<'a>(&'a self) -> WebResult<std::borrow::Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(&self.0.to_be_bytes()))
    }
}

#[inline(always)]
pub fn read_u64<T: Buf + MarkBuf>(buf: &mut T) -> u64 {
    if buf.remaining() < 8 {
        return 0;
    }
    buf.get_u64()
}

pub const MASK_U31: u32 = (1u32 << 31) - 1;
#[inline(always)]
pub fn read_u31<T: Buf + MarkBuf>(buf: &mut T) -> u32 {
    if buf.remaining() < 4 {
        return 0;
    }
    let val = buf.get_u32();
    val & MASK_U31
    // (buf[0] as u32 & 0x7F) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | buf[3] as u32
}

#[inline(always)]
pub fn read_u24<T: Buf + MarkBuf>(buf: &mut T) -> u32 {
    if buf.remaining() < 3 {
        return 0;
    }
    (buf.get_u8() as u32) << 16 | (buf.get_u8() as u32) << 8 | buf.get_u8() as u32
}

#[inline(always)]
pub fn encode_u24<B: Buf + BufMut + MarkBuf>(buf: &mut B, val: u32) -> usize {
    buf.put_u8((val >> 16) as u8);
    buf.put_u8((val >> 8) as u8);
    buf.put_u8((val >> 0) as u8);
    3
}

#[inline(always)]
pub fn encode_u32(buf: &mut [u8], val: u32) -> usize {
    buf[0] = (val >> 24) as u8;
    buf[1] = (val >> 16) as u8;
    buf[2] = (val >> 8) as u8;
    buf[3] = val as u8;
    4
}

#[inline(always)]
pub fn encode_u64(buf: &mut [u8], val: u64) -> usize {
    encode_u32(buf, (val >> 16) as u32);
    encode_u32(&mut buf[4..], val as u32);
    8
}
pub struct Http2;

impl Http2 {
    pub fn parse_buffer<T: serialize::Serialize, B: Buf + MarkBuf>(
        request: &mut Request<T>,
        buffer: &mut B,
    ) -> WebResult<()> {
        while buffer.has_remaining() {
            let frame_header = FrameHeader::parse(buffer)?;
            let frame = Frame::parse(frame_header, buffer)?;
            println!("frame = {:?}", frame);
            match frame.payload {
                Payload::Data { data } => {
                    
                }
                Payload::Headers { priority, mut block } => {
                    request.parse_http2_header(&mut block)?;
                    if request.method().is_nobody() {
                        return Ok(());
                    }
                }
                Payload::Priority(priority) => {

                }
                Payload::Reset(err) => {
                    
                }
                Payload::Settings(s) => {
                    
                }
                Payload::WindowUpdate(s) => {
                    
                }
                _ => {

                }
            }
            
        }
        Ok(())
    }

    pub fn build_body_frame<T: serialize::Serialize>(res: &mut Response<T>) -> WebResult<Option<Frame<BinaryMut>>> {
        let mut buf = BinaryMut::new();
        res.body().serialize(&mut buf)?;
        if buf.remaining() == 0 {
            return Ok(None)
        }
        let header = FrameHeader {
            length: buf.remaining() as u32,
            kind: Kind::Data,
            flag: Flag::end_stream(),
            id: StreamIdentifier(4),
        };
        let payload = Payload::Data { data: buf };
        let frame = Frame {
            header,
            payload,
        };
        Ok(Some(frame))
    }

    
    pub fn build_header_frame<T: serialize::Serialize>(res: &mut Response<T>) -> WebResult<Frame<BinaryMut>> {
        let mut buf = BinaryMut::new();
        let mut enocder = res.get_encoder();
        let status = res.status().build_header();
        enocder.encode_header_into((&status.0, &status.1), &mut buf).map_err(WebError::from)?;
        enocder.encode_into(res.headers().iter(), &mut buf)?;
        let header = FrameHeader {
            length: buf.remaining() as u32,
            kind: Kind::Headers,
            flag: Flag::end_headers(),
            id: StreamIdentifier(2),
        };
        let payload = Payload::Headers { priority: None, block: buf };
        let frame = Frame {
            header,
            payload,
        };
        Ok(frame)
    }

    pub fn build_response_frame<T: serialize::Serialize>(res: &mut Response<T>) -> WebResult<Vec<Frame<BinaryMut>>>  {
        let mut result = vec![];
        result.push(Self::build_header_frame(res)?);
        if let Some(frame) = Self::build_body_frame(res)? {
            result.push(frame);
        }
        Ok(result)
    }

    pub fn serialize<T: serialize::Serialize>(res: &mut Response<T>, buffer: &mut BinaryMut) -> WebResult<()> {
        let vecs = Self::build_response_frame(res)?;
        for vec in vecs {
            vec.serialize(buffer);
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub fn parse<T: Buf + MarkBuf>(buf: &mut T) -> ErrorCode {
        buf.advance(4);
        ErrorCode(0)
    }

    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        buf.put_u32(self.0);
        4
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SizeIncrement(pub u32);

impl SizeIncrement {
    pub fn parse<T: Buf + MarkBuf>(buf: &mut T) -> SizeIncrement {
        buf.advance(4);
        SizeIncrement(0)
    }

    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        buf.put_u32(self.0);
        4
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserSettings {
    padding: bool,
    priority: bool,
}
