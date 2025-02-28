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

use std::fmt;

use crate::{Http2Error, WebResult};
use algorithm::buf::{Binary, Bt, BtMut};

use super::{frame, Flag, FrameHeader, Kind, Reason, StreamIdentifier};

#[derive(Clone, Eq, PartialEq)]
pub struct GoAway {
    last_stream_id: StreamIdentifier,
    error_code: Reason,
    debug_data: Binary,
}

impl GoAway {
    pub fn new(last_stream_id: StreamIdentifier, reason: Reason) -> Self {
        GoAway {
            last_stream_id,
            error_code: reason,
            debug_data: Binary::new(),
        }
    }

    pub fn with_debug_data(
        last_stream_id: StreamIdentifier,
        reason: Reason,
        debug_data: Binary,
    ) -> Self {
        Self {
            last_stream_id,
            error_code: reason,
            debug_data,
        }
    }

    pub fn last_stream_id(&self) -> StreamIdentifier {
        self.last_stream_id
    }

    pub fn reason(&self) -> Reason {
        self.error_code
    }

    pub fn debug_data(&self) -> &Binary {
        &self.debug_data
    }

    pub fn parse<B: Bt>(payload: &mut B) -> WebResult<GoAway> {
        if payload.remaining() < 8 {
            return Err(Http2Error::BadFrameSize.into());
        }

        let last_stream_id = StreamIdentifier::parse(payload);
        let error_code = payload.get_u32();
        let debug_data = Binary::copy_from_slice(&payload.chunk());

        Ok(GoAway {
            last_stream_id,
            error_code: error_code.into(),
            debug_data,
        })
    }

    pub(crate) fn head(&self) -> FrameHeader {
        let mut head = FrameHeader::new(Kind::GoAway, Flag::zero(), StreamIdentifier::zero());
        head.length = 8 + self.debug_data.remaining() as u32;
        head
    }

    pub fn encode<B: Bt + BtMut>(&self, buffer: &mut B) -> crate::WebResult<usize> {
        let mut size = 0;
        size += self.head().encode(buffer)?;
        size += buffer.put_u32(self.last_stream_id.0);
        size += buffer.put_u32(self.error_code.into());
        size += buffer.put_slice(self.debug_data.chunk());
        Ok(size)
    }
}

impl<B> From<GoAway> for frame::Frame<B> {
    fn from(src: GoAway) -> Self {
        frame::Frame::GoAway(src)
    }
}

impl fmt::Debug for GoAway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("GoAway");
        builder.field("error_code", &self.error_code);
        builder.field("last_stream_id", &self.last_stream_id);

        if !self.debug_data.is_empty() {
            builder.field("debug_data", &self.debug_data);
        }

        builder.finish()
    }
}
