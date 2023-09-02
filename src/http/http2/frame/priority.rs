use crate::{WebResult, Http2Error, http::http2::Http2, Buf};

use super::{StreamIdentifier, FrameHeader, frame::Frame};

#[derive(Debug, Eq, PartialEq)]
pub struct Priority {
    stream_id: StreamIdentifier,
    dependency: StreamDependency,
}

#[derive(Debug, Eq, PartialEq)]
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

        // Parse the stream ID and exclusive flag
        let dependency_id = StreamIdentifier::parse(src);

        // Read the weight
        let weight = src.get_u8();
        // todo!!
        Ok(StreamDependency::new(dependency_id, weight, false))
    }

    pub fn dependency_id(&self) -> StreamIdentifier {
        self.dependency_id
    }
}
