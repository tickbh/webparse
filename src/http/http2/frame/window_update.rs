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
// Created Date: 2023/09/01 04:39:00

use algorithm::buf::{Bt, BtMut};
use crate::{http::http2::frame::Kind, Http2Error, WebResult};

use super::{StreamIdentifier, FrameHeader, frame::Frame, Flag};



const SIZE_INCREMENT_MASK: u32 = 1 << 31;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WindowUpdate {
    stream_id: StreamIdentifier,
    size_increment: u32,
}

impl WindowUpdate {
    pub fn new(stream_id: StreamIdentifier, size_increment: u32) -> WindowUpdate {
        WindowUpdate {
            stream_id,
            size_increment,
        }
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn size_increment(&self) -> u32 {
        self.size_increment
    }

    /// Builds a `WindowUpdate` frame from a raw frame.
    pub fn parse<B: Bt>(head: FrameHeader, payload: &mut B) -> WebResult<WindowUpdate> {
        debug_assert_eq!(head.kind(), &Kind::WindowUpdate);
        if payload.remaining() != 4 {
            return Err(Http2Error::BadFrameSize.into());
        }

        // Clear the most significant bit, as that is reserved and MUST be ignored
        // when received.
        let size_increment = payload.get_u32() & !SIZE_INCREMENT_MASK;

        if size_increment == 0 {
            return Err(Http2Error::InvalidWindowUpdateValue.into());
        }

        Ok(WindowUpdate {
            stream_id: head.stream_id(),
            size_increment,
        })
    }

    
    pub(crate) fn head(&self) -> FrameHeader {
        let mut head = FrameHeader::new(Kind::WindowUpdate, Flag::zero(), self.stream_id);
        head.length = 4;
        head
    }

    pub fn encode<B: Bt+BtMut>(&self, buffer: &mut B) -> crate::WebResult<usize> {
        let mut size = 0;
        size += self.head().encode(buffer)?;
        size += buffer.put_u32(self.size_increment);
        log::trace!("encoding WindowUpdate; len={}", size);
        log::trace!("HTTP2: 编码窗口更新信息; len={}", size);
        Ok(size)
    }

}



impl<B> From<WindowUpdate> for Frame<B> {
    fn from(src: WindowUpdate) -> Self {
        Frame::WindowUpdate(src)
    }
}
