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
// Created Date: 2023/09/01 04:44:01

use algorithm::buf::{Bt, BtMut};
use crate::{WebResult, Http2Error};

use super::{StreamIdentifier, Reason, FrameHeader, frame::Frame, Kind, Flag};


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Reset {
    stream_id: StreamIdentifier,
    error_code: Reason,
}

impl Reset {
    pub fn new(stream_id: StreamIdentifier, error: Reason) -> Reset {
        Reset {
            stream_id,
            error_code: error,
        }
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn reason(&self) -> Reason {
        self.error_code
    }

    pub fn parse<B: Bt>(head: FrameHeader, payload: &mut B) -> WebResult<Reset> {
        if payload.remaining() != 4 {
            return Err(Http2Error::InvalidPayloadLength.into());
        }

        let error_code = payload.get_u32();

        Ok(Reset {
            stream_id: head.stream_id(),
            error_code: error_code.into(),
        })
    }

    pub(crate) fn head(&self) -> FrameHeader {
        let mut head = FrameHeader::new(Kind::Reset, Flag::zero(), self.stream_id);
        head.length = 4;
        head
    }

    pub fn encode<B: Bt+BtMut>(&self, buffer: &mut B) -> crate::WebResult<usize> {
        let mut size = 0;
        size += self.head().encode(buffer)?;
        size += buffer.put_u32(self.error_code.into());
        log::trace!("HTTP2: 编码Reset信息; len={}", size);
        Ok(size)
    }
}



impl<B> From<Reset> for Frame<B> {
    fn from(src: Reset) -> Self {
        Frame::Reset(src)
    }
}
