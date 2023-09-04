use std::fmt::Debug;

use crate::{
    http::http2::{Decoder, encoder::Encoder}, Binary, Buf, BufMut, HeaderMap, Http2Error, MarkBuf,
    Serialize, WebResult,
};

use super::{
    encode_u24,
    headers::{Continuation, PushPromise},
    read_u24, Data, Flag, GoAway, Headers, Kind, Ping, Priority, Reset, Settings, StreamIdentifier,
    WindowUpdate,
};

pub const FRAME_HEADER_BYTES: usize = 9;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameHeader {
    pub length: u32,
    pub kind: Kind,
    pub flag: Flag,
    pub id: StreamIdentifier,
}

#[derive(Debug)]
pub enum Frame<T = Binary> {
    Data(Data<T>),
    Headers(Headers),
    Priority(Priority),
    PushPromise(PushPromise),
    Settings(Settings),
    Ping(Ping),
    GoAway(GoAway),
    WindowUpdate(WindowUpdate),
    Reset(Reset),
}

impl Frame<Binary> {
    #[inline]
    pub fn trim_padding<B: Buf + MarkBuf>(header: &FrameHeader, buf: &mut B) -> WebResult<()> {
        if header.flag.is_padded() && buf.has_remaining() {
            let pad_length = buf.peek().unwrap();
            if pad_length as u32 > header.length {
                return Err(Http2Error::into(Http2Error::TooMuchPadding(pad_length)));
            } else {
                buf.advance(1);
                buf.mark_len(header.length as usize - pad_length as usize - 1);
            }
        }
        Ok(())
    }
}

impl<T: Buf + MarkBuf> Frame<T> {
    pub fn parse(
        header: FrameHeader,
        mut buf: T,
        decoder: &mut Decoder,
        max_header_list_size: usize,
    ) -> WebResult<Frame<T>> {
        Frame::trim_padding(&header, &mut buf)?;
        match header.kind() {
            Kind::Data => Ok(Frame::Data(Data::new(header.stream_id(), buf))),
            Kind::Headers => {
                let mut header = Headers::new(header.stream_id(), HeaderMap::new());
                header.parse(buf, decoder, max_header_list_size)?;
                Ok(Frame::Headers(header))
            }
            Kind::Priority => Ok(Frame::Priority(Priority::parse(header, &mut buf)?)),
            Kind::Reset => Ok(Frame::Reset(Reset::parse(header, &mut buf)?)),
            Kind::Settings => Ok(Frame::Settings(Settings::parse(header, &mut buf)?)),
            Kind::PushPromise => Ok(Frame::PushPromise(PushPromise::parse(
                header,
                buf,
                decoder,
                max_header_list_size,
            )?)),
            Kind::Ping => Ok(Frame::Ping(Ping::parse(header, &mut buf)?)),
            Kind::GoAway => Ok(Frame::GoAway(GoAway::parse(&mut buf)?)),
            Kind::WindowUpdate => Ok(Frame::WindowUpdate(WindowUpdate::parse(header, &mut buf)?)),
            Kind::Continuation => {
                Err(crate::WebError::Extension(""))
                // Ok(Frame::Continuation(Continuation::parse(header, &mut buf)?))
            }
            _ => Err(crate::WebError::Extension("")),
        }
    }

    pub fn encode<B: Buf+MarkBuf+BufMut>(
        mut self,
        buf: &mut B,
        encoder: &mut Encoder
    ) -> WebResult<usize> {
        let size = match self {
            Frame::Data(_) => todo!(),
            Frame::Headers(_) => todo!(),
            Frame::Priority(_) => todo!(),
            Frame::PushPromise(_) => todo!(),
            Frame::Settings(s) => s.encode(buf),
            Frame::Ping(_) => todo!(),
            Frame::GoAway(_) => todo!(),
            Frame::WindowUpdate(_) => todo!(),
            Frame::Reset(_) => todo!(),
        };
        Ok(size)
    }

    /// How many bytes this Frame will use in a buffer when encoding.
    pub fn encoded_len(&self) -> usize {
        0
        // FRAME_HEADER_BYTES + self.payload.encoded_len()
    }

    pub fn no_serialize_header(&self) -> bool {
        false
        // if self.header.kind == Kind::Settings {
        //     true
        // } else {
        //     false
        // }
    }
}

impl<T: Buf + MarkBuf> Serialize for Frame<T> {
    fn serialize<B: Buf + BufMut + MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        // if !self.no_serialize_header() {
        //     size += self.header.serialize(buffer)?;
        // }
        // size += self.payload.serialize(buffer)?;
        Ok(size)
    }
}

impl FrameHeader {
    pub fn new(kind: Kind, flag: Flag, id: StreamIdentifier) -> FrameHeader {
        FrameHeader {
            length: 0,
            kind,
            flag,
            id,
        }
    }
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

    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.id
    }

    pub fn flag(&self) -> Flag {
        self.flag
    }
}

impl Serialize for FrameHeader {
    fn serialize<B: Buf + BufMut + MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += encode_u24(buffer, self.length);
        size += buffer.put_u8(self.kind.encode());
        size += buffer.put_u8(self.flag.bits());
        size += self.id.serialize(buffer)?;
        Ok(size)
    }
}

// impl<T: Buf + MarkBuf> Debug for Frame<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Frame")
//             // .field("header", &self.header)
//             // .field("payload", &self.payload)
//             .finish()
//     }
// }

// impl<T> Ord for Frame<T> {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.partial_cmp(&other).unwrap()
//     }
// }

pub struct PriorityFrame<T = Binary> {
    pub frame: Frame<T>,
    pub weight: u8,
}

impl<T> PriorityFrame<T> {
    pub fn new(frame: Frame<T>) -> Self {
        Self { frame, weight: 0 }
    }

    pub fn set_weight(&mut self, weight: u8) {
        self.weight = weight;
    }

    pub fn weight(&self) -> u8 {
        self.weight
    }
}

impl<T> Ord for PriorityFrame<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl<T> PartialOrd for PriorityFrame<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}
impl<T> Eq for PriorityFrame<T> {}

impl<T> PartialEq for PriorityFrame<T> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
