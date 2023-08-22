pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";


mod flag;
mod kind;
mod frame;
mod payload;
mod error;
// mod hpack;

pub use frame::Frame;
pub use payload::Payload;
pub use flag::Flag;
pub use kind::Kind;
pub use error::Http2Error;
// pub use hpack::*;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse(buf: &[u8]) -> StreamIdentifier {
        if buf.len() < 4 {
            return StreamIdentifier(0);
        }
        StreamIdentifier(
            read_u31(buf)
        )
    }

    pub fn encode(&self, buf: &mut [u8]) -> usize {
        encode_u32(buf, self.0)
    }
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