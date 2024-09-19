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
// Created Date: 2023/09/01 04:35:19

use crate::{Http2Error, WebResult};
use algorithm::buf::{Bt, BtMut};

use super::{frame::Frame, Flag, FrameHeader, StreamIdentifier, MASK_U31};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Priority {
    stream_id: StreamIdentifier,
    dependency: StreamDependency,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StreamDependency {
    /// The ID of the stream dependency target
    dependency_id: StreamIdentifier,

    /// The weight for the stream. The value exposed (and set) here is always in
    /// the range [0, 255], instead of [1, 256] (as defined in section 5.3.2.)
    /// so that the value fits into a `u8`.
    weight: u8,

    /// True if the stream dependency is exclusive.
    is_exclusive: bool,
}

impl Priority {
    pub fn parse<B: Bt>(head: FrameHeader, payload: &mut B) -> WebResult<Self> {
        let dependency = StreamDependency::load(payload)?;

        if dependency.dependency_id() == head.stream_id() {
            return Err(Http2Error::InvalidDependencyId.into());
        }

        Ok(Priority {
            stream_id: head.stream_id(),
            dependency,
        })
    }

    pub fn into(self) -> (StreamIdentifier, StreamIdentifier, u8) {
        (
            self.stream_id,
            self.dependency.dependency_id,
            self.dependency.weight,
        )
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn dependency_id(&self) -> StreamIdentifier {
        self.dependency.dependency_id
    }

    pub fn weight(&self) -> u8 {
        self.dependency.weight
    }

    pub fn encode<B: Bt + BtMut>(&self, dst: &mut B) -> WebResult<usize> {
        let head = FrameHeader::new(super::Kind::Priority, Flag::zero(), self.stream_id);
        let mut size = 0;
        size += head.encode(dst)?;
        size += self.dependency.encode(dst)?;
        log::trace!("HTTP2: 编码优先级信息; len={}", size);
        Ok(size)
    }
}

impl<B> From<Priority> for Frame<B> {
    fn from(src: Priority) -> Self {
        Frame::Priority(src)
    }
}

// ===== impl StreamDependency =====

impl StreamDependency {
    pub fn new(dependency_id: StreamIdentifier, weight: u8, is_exclusive: bool) -> Self {
        StreamDependency {
            dependency_id,
            weight,
            is_exclusive,
        }
    }

    pub fn load<B: Bt>(src: &mut B) -> WebResult<Self> {
        if src.remaining() < 5 {
            return Err(Http2Error::InvalidPayloadLength.into());
        }

        let value = src.get_u32();
        let id = value & MASK_U31;
        let is_exclusive = value - id != 0;

        let dependency_id = StreamIdentifier(id);
        let weight = src.get_u8();
        Ok(StreamDependency::new(dependency_id, weight, is_exclusive))
    }

    pub fn dependency_id(&self) -> StreamIdentifier {
        self.dependency_id
    }

    fn encode<B: Bt + BtMut>(&self, dst: &mut B) -> WebResult<usize> {
        self.dependency_id.encode(dst)?;
        dst.put_u8(self.weight);
        Ok(5)
    }
}
