use crate::{WebResult, Http2Error, Buf, BufMut};

use super::{StreamIdentifier, FrameHeader, frame::Frame, Flag};

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
    pub fn parse<B: Buf>(head: FrameHeader, payload: &mut B) -> WebResult<Self> {
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
        (self.stream_id, self.dependency.dependency_id, self.dependency.weight)
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

    pub fn encode<B: Buf + BufMut>(&self, dst: &mut B) -> WebResult<usize> {
        let head = FrameHeader::new(super::Kind::Priority, Flag::zero(), self.stream_id);
        log::trace!("encoding Priority; len={}", 5);
        let mut size = 0;
        size += head.encode(dst)?;
        size += self.dependency.encode(dst)?;
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

    pub fn load<B: Buf>(src: &mut B) -> WebResult<Self> {
        if src.remaining() != 5 {
            return Err(Http2Error::InvalidPayloadLength.into());
        }

        let dependency_id = StreamIdentifier::parse(src);
        let weight = src.get_u8();
        Ok(StreamDependency::new(dependency_id, weight, false))
    }

    pub fn dependency_id(&self) -> StreamIdentifier {
        self.dependency_id
    }
    
    fn encode<B: Buf + BufMut>(&self, dst: &mut B) -> WebResult<usize> {
        self.dependency_id.encode(dst)?;
        dst.put_u8(self.weight);
        Ok(5)
    }
}
