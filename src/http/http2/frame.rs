use std::fmt::Debug;

use crate::{Buf, BufMut, Http2Error, MarkBuf, WebResult, Serialize};

use super::{encode_u24, read_u24, Flag, Kind, Payload, StreamIdentifier};

pub const FRAME_HEADER_BYTES: usize = 9;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Frame<T: Buf + MarkBuf> {
    pub header: FrameHeader,
    pub payload: Payload<T>,
}

impl<T: Buf + MarkBuf> Frame<T> {
    pub fn parse(header: FrameHeader, buf: &mut T) -> WebResult<Frame<T>> {
        Ok(Frame {
            header: header,
            payload: Payload::parse(header, buf)?,
        })
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        FRAME_HEADER_BYTES + self.payload.encoded_len()
    }
}


impl<T: Buf + MarkBuf> Serialize for Frame<T> {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += self.header.serialize(buffer)?;
        size += self.payload.serialize(buffer)?;
        Ok(size)
    }
}

impl FrameHeader {
    #[inline]
    pub fn parse<T: Buf + MarkBuf>(buffer: &mut T) -> WebResult<FrameHeader> {
        if buffer.remaining() < FRAME_HEADER_BYTES {
            return Err(Http2Error::into(Http2Error::Short));
        }
        let length = read_u24(buffer);
        let kind = Kind::new(buffer.get_u8());
        let flag = buffer.get_u8();
        let id = StreamIdentifier::parse(buffer);
        Ok(FrameHeader {
            length,
            kind,
            flag: Flag::new(flag).map_err(|()| Http2Error::into(Http2Error::BadFlag(flag)))?,
            id,
        })
    }

}


impl Serialize for FrameHeader {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += encode_u24(buffer, self.length);
        size += buffer.put_u8(self.kind.encode());
        size += buffer.put_u8(self.flag.bits());
        size += self.id.serialize(buffer)?;
        Ok(size)
    }
}

impl<T: Buf + MarkBuf> Debug for Frame<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Frame")
            .field("header", &self.header)
            // .field("payload", &self.payload)
            .finish()
    }
}
