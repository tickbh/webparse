pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

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

use crate::{serialize, Request, WebResult, BinaryMut, Buf, MarkBuf, http::http2::frame::FRAME_HEADER_BYTES, BufMut};
pub use hpack::*;

use self::frame::FrameHeader;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse<T: Buf+MarkBuf>(buf: &mut T) -> StreamIdentifier {
        if buf.remaining() < 4 {
            return StreamIdentifier(0);
        }
        StreamIdentifier(read_u31(buf))
    }

    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        buf.put_u32(self.0);
        4
    }
}

#[inline(always)]
pub fn read_u64<T:Buf+MarkBuf>(buf: &mut T) -> u64 {
    if buf.remaining() < 8 {
        return 0;
    }
    buf.get_u64()
}

pub const MASK_U31: u32 = (1u32 << 31) - 1;
#[inline(always)]
pub fn read_u31<T: Buf+MarkBuf>(buf: &mut T) -> u32 {
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
            buffer.advance(FRAME_HEADER_BYTES);
            let length = frame_header.length;
            {
                let frame = Frame::parse(frame_header, buffer)?;
                // println!("frame = {:?}", frame);
            }
            // buffer.advance(length as usize);
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
    pub fn parse<T: Buf+MarkBuf>(buf: &mut T) -> SizeIncrement {
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
