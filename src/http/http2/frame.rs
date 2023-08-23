use crate::{WebResult, Http2Error};

use super::{Kind, Payload, StreamIdentifier, Flag, encode_u24};

pub const FRAME_HEADER_BYTES: usize = 9;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Frame<'a> {
    pub header: FrameHeader,
    pub payload: Payload<'a>
}


impl<'a> Frame<'a> {
    pub fn parse(header: FrameHeader, buf: &[u8]) -> WebResult<Frame> {
        Ok(Frame {
            header: header,
            payload: Payload::parse(header, buf)?
        })
    }

    /// Encodes this Frame into a buffer.
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        self.header.encode(buf);
        self.payload.encode(&mut buf[FRAME_HEADER_BYTES..]) + FRAME_HEADER_BYTES
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        FRAME_HEADER_BYTES + self.payload.encoded_len()
    }
}


impl FrameHeader {
    #[inline]
    pub fn parse(buf: &[u8]) -> WebResult<FrameHeader> {
        if buf.len() < FRAME_HEADER_BYTES {
            return Err(Http2Error::into(Http2Error::Short));
        }

        Ok(FrameHeader {
            length: ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | buf[2] as u32,
            kind: Kind::new(buf[3]),
            flag: Flag::new(buf[4]).map_err(|()| { Http2Error::into(Http2Error::BadFlag(buf[4])) })?,
            id: StreamIdentifier::parse(&buf[5..])
        })
    }

    #[inline]
    pub fn encode(&self, buf: &mut [u8]) {
        encode_u24(buf, self.length);
        buf[3] = self.kind.encode();
        buf[4] = self.flag.bits();
        self.id.encode(&mut buf[5..]);
    }
}