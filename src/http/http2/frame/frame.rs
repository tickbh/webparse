// Copyright 2022 - 2023 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Author: tickbh
// -----
// Created Date: 2023/08/21 11:20:39

use std::fmt::Debug;

use crate::{
    http::http2::{encoder::Encoder, Decoder},
    HeaderMap, Http2Error, Serialize, WebResult,
};
use algorithm::buf::{Binary, Bt, BtMut};

use super::{
    encode_u24, headers::PushPromise, read_u24, Data, Flag, GoAway, Headers, Kind, Ping, Priority,
    Reset, Settings, StreamIdentifier, WindowUpdate,
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
    pub fn trim_padding<B: Bt>(header: &FrameHeader, buf: &mut B) -> WebResult<()> {
        if header.flag.is_padded() && buf.has_remaining() {
            let pad_length = buf.peek().unwrap();
            if pad_length as u32 > header.length {
                return Err(Http2Error::into(Http2Error::TooMuchPadding(pad_length)));
            } else {
                buf.advance(1);
                // buf.mark_len(header.length as usize - pad_length as usize - 1);
            }
        }
        Ok(())
    }

    pub fn display_name(&self) -> String {
        match self {
            Frame::Data(f) => format!("Data({})", f.stream_id()),
            Frame::Headers(f) => format!("Headers({})", f.stream_id()),
            Frame::Priority(f) => format!("Priority({})", f.stream_id()),
            Frame::PushPromise(f) => format!("PushPromise({})", f.stream_id()),
            Frame::Settings(_f) => format!("Settings({})", 0),
            Frame::Ping(_f) => format!("Ping({})", 0),
            Frame::GoAway(_f) => format!("GoAway({})", 0),
            Frame::WindowUpdate(f) => format!("WindowUpdate({})", f.stream_id()),
            Frame::Reset(f) => format!("Reset({})", f.stream_id()),
        }
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        match self {
            Frame::Data(f) => f.stream_id(),
            Frame::Headers(f) => f.stream_id(),
            Frame::Priority(_f) => StreamIdentifier::zero(),
            Frame::PushPromise(f) => f.stream_id(),
            Frame::Settings(_f) => StreamIdentifier::zero(),
            Frame::Ping(_f) => StreamIdentifier::zero(),
            Frame::GoAway(_f) => StreamIdentifier::zero(),
            Frame::WindowUpdate(f) => f.stream_id(),
            Frame::Reset(f) => f.stream_id(),
        }
    }

    pub fn flags(&self) -> Flag {
        match self {
            Frame::Data(f) => f.flags(),
            Frame::Headers(f) => f.flags(),
            Frame::Priority(_f) => Flag::zero(),
            Frame::PushPromise(f) => f.flags(),
            Frame::Settings(f) => f.flags(),
            Frame::Ping(_f) => Flag::zero(),
            Frame::GoAway(_f) => Flag::zero(),
            Frame::WindowUpdate(_f) => Flag::zero(),
            Frame::Reset(_f) => Flag::zero(),
        }
    }

    pub fn is_header(&self) -> bool {
        match self {
            Frame::Headers(_) => true,
            _ => false,
        }
    }

    pub fn is_data(&self) -> bool {
        match self {
            Frame::Data(_) => true,
            _ => false,
        }
    }

    pub fn is_end_headers(&self) -> bool {
        match self {
            Frame::Headers(f) => f.is_end_headers(),
            _ => false,
        }
    }

    pub fn is_end_stream(&self) -> bool {
        match self {
            Frame::Headers(f) => f.is_end_stream(),
            Frame::Data(f) => f.is_end_stream(),
            // Frame::PushPromise(f) => f.is_end_stream(),
            _ => false,
        }
    }

    pub fn encode<B: Bt + BtMut>(self, buf: &mut B, encoder: &mut Encoder) -> WebResult<usize> {
        let name = self.display_name();
        let size = match self {
            Frame::Data(mut s) => s.encode(encoder, buf)?,
            Frame::Headers(s) => s.encode(encoder, buf)?,
            Frame::Priority(v) => v.encode(buf)?,
            Frame::PushPromise(p) => p.encode(encoder, buf)?,
            Frame::Settings(s) => s.encode(buf)?,
            Frame::Ping(v) => v.encode(buf)?,
            Frame::GoAway(v) => v.encode(buf)?,
            Frame::WindowUpdate(v) => v.encode(buf)?,
            Frame::Reset(v) => v.encode(buf)?,
        };
        log::trace!("编码http2二进制Frame({}) 大小 {}", name, size);
        Ok(size)
    }
}

impl<T: Bt> Frame<T> {
    pub fn parse(
        header: FrameHeader,
        mut buf: T,
        decoder: &mut Decoder,
        max_header_list_size: usize,
    ) -> WebResult<Frame<T>> {
        Frame::trim_padding(&header, &mut buf)?;
        match header.kind() {
            Kind::Data => Ok(Frame::Data(Data::new(header, buf))),
            Kind::Headers => {
                let mut header = Headers::new(header, HeaderMap::new());
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

impl<T: Bt> Serialize for Frame<T> {
    fn serialize<B: Bt + BtMut>(&mut self, _buffer: &mut B) -> WebResult<usize> {
        let size = 0;
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
    pub fn parse<T: Bt>(buffer: &mut T) -> WebResult<FrameHeader> {
        if buffer.remaining() < FRAME_HEADER_BYTES {
            return Err(Http2Error::into(Http2Error::Short));
        }
        let length = read_u24(buffer);
        let kind = Kind::new(buffer.get_u8());
        let flag = buffer.get_u8();
        let flag = Flag::new(flag).map_err(|()| Http2Error::into(Http2Error::BadFlag(flag)))?;
        let id = StreamIdentifier::parse(buffer);
        Ok(FrameHeader {
            length,
            kind,
            flag,
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

    pub fn flags_mut(&mut self) -> &mut Flag {
        &mut self.flag
    }

    pub fn encode<B: Bt + BtMut>(&self, buffer: &mut B) -> WebResult<usize> {
        let mut size = 0;
        size += encode_u24(buffer, self.length);
        size += buffer.put_u8(self.kind.encode());
        size += buffer.put_u8(self.flag.bits());
        size += self.id.encode(buffer)?;
        Ok(size)
    }
}

// impl<T: Bt> Debug for Frame<T> {
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

#[derive(Debug)]
pub struct PriorityFrame<T = Binary> {
    pub frame: Frame<T>,
    pub weight: u8,
}

impl<T> PriorityFrame<T> {
    pub fn new(frame: Frame<T>, weight: u8) -> Self {
        Self { frame, weight }
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
