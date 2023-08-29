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

use crate::{serialize, Request, WebResult, BinaryMut};
pub use hpack::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        if buf.len() < 4 {
            return StreamIdentifier(0);
        }
        StreamIdentifier(read_u31(buf))
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[inline(always)]
pub fn read_u64(buf: &[u8]) -> u64 {
    if buf.len() < 8 {
        return 0;
    }
    (buf[0] as u64 & 0x7F) << 56
        | (buf[1] as u64) << 48
        | (buf[2] as u64) << 40
        | (buf[3] as u64) << 32
        | (buf[5] as u64 & 0x7F) << 24
        | (buf[6] as u64) << 16
        | (buf[7] as u64) << 8
        | buf[8] as u64
}

#[inline(always)]
pub fn read_u31(buf: &[u8]) -> u32 {
    if buf.len() < 4 {
        return 0;
    }
    (buf[0] as u32 & 0x7F) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | buf[3] as u32
}

#[inline(always)]
pub fn read_u24(buf: &[u8]) -> u32 {
    if buf.len() < 3 {
        return 0;
    }
    (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | buf[3] as u32
}

#[inline(always)]
pub fn encode_u24(buf: &mut [u8], val: u32) -> usize {
    buf[0] = (val >> 16) as u8;
    buf[1] = (val >> 8) as u8;
    buf[2] = val as u8;
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
    pub fn parse_buffer<T: serialize::Serialize>(
        request: &mut Request<T>,
        buffer: &mut BinaryMut,
    ) -> WebResult<()> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub fn parse(buf: &[u8]) -> ErrorCode {
        ErrorCode(0)
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SizeIncrement(pub u32);

impl SizeIncrement {
    pub fn parse(buf: &[u8]) -> SizeIncrement {
        SizeIncrement(0)
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserSettings {
    padding: bool,
    priority: bool,
}
