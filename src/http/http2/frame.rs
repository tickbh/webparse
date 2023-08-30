use crate::{WebResult, Http2Error, MarkBuf, Buf, BufMut};

use super::{Kind, Payload, StreamIdentifier, Flag, encode_u24, read_u24};

pub const FRAME_HEADER_BYTES: usize = 9;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame<T: Buf+MarkBuf> {
    pub header: FrameHeader,
    pub payload: Payload<T>
}


impl<T:Buf+MarkBuf> Frame<T> {
    pub fn parse(header: FrameHeader, buf: &mut T) -> WebResult<Frame<T>> {
        Ok(Frame {
            header: header,
            payload: Payload::parse(header, buf)?
        })
    }

    /// Encodes this Frame into a buffer.
    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) -> usize {
        self.header.encode(buf);
        // self.payload.encode(&mut buf[FRAME_HEADER_BYTES..]) + FRAME_HEADER_BYTES
        0
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        FRAME_HEADER_BYTES + self.payload.encoded_len()
    }
}


impl FrameHeader {
    #[inline]
    pub fn parse<T:Buf+MarkBuf>(buffer: &mut T) -> WebResult<FrameHeader> {
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
            flag: Flag::new(flag).map_err(|()| { Http2Error::into(Http2Error::BadFlag(flag)) })?,
            id
        })
    }

    #[inline]
    pub fn encode<B: Buf + BufMut + MarkBuf>(&self, buf: &mut B) {
        encode_u24(buf, self.length);
        buf.put_u8(self.kind.encode());
        buf.put_u8(self.flag.bits());
        self.id.encode(buf);
    }
}