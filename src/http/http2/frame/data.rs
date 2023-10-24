
use crate::{Binary, Serialize, Buf, BufMut, WebResult, http2::encoder::Encoder};

use super::{Flag, FrameHeader, Kind, StreamIdentifier};

#[derive(Eq, PartialEq, Debug)]
pub struct Data<T = Binary> {
    stream_id: StreamIdentifier,
    data: T,
    flags: Flag,
    pad_len: Option<u8>,
}

impl<T> Data<T> {
    pub fn new(header: FrameHeader, payload: T) -> Self {
        assert!(!header.stream_id().is_zero());

        Data {
            stream_id: header.stream_id(),
            data: payload,
            flags: header.flag(),
            pad_len: None,
        }
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn is_end_stream(&self) -> bool {
        self.flags.is_end_stream()
    }

    pub fn set_end_stream(&mut self, val: bool) {
        if val {
            self.flags.set_end_stream();
        } else {
            self.flags.unset_end_stream();
        }
    }

    pub fn flags(&self) -> Flag {
        self.flags
    }

    pub fn is_padded(&self) -> bool {
        self.flags.is_padded()
    }

    pub fn set_padded(&mut self) {
        self.flags.set_padded();
    }

    pub fn payload(&self) -> &T {
        &self.data
    }

    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn into_payload(self) -> T {
        self.data
    }

    pub fn map<F, U>(self, f: F) -> Data<U>
    where
        F: FnOnce(T) -> U,
    {
        Data {
            stream_id: self.stream_id,
            data: f(self.data),
            flags: self.flags,
            pad_len: self.pad_len,
        }
    }
}

impl Data<Binary> {
    pub fn encode<B: Buf+BufMut>(&mut self,
        encoder: &mut Encoder, dst: &mut B) -> WebResult<usize> {
        log::trace!("encoding Data; len={}", self.data.remaining());
        let mut size = 0;
        loop {
            let now_len = std::cmp::min(self.data.remaining(), encoder.max_frame_size); 
            let mut head = FrameHeader::new(Kind::Data, self.flags.into(), self.stream_id);
            head.length = now_len as u32;
            if now_len < self.data.remaining() {
                head.flags_mut().unset_end_stream();
                size += head.encode(dst)?;
                size += dst.put_slice(&self.data.chunk()[..now_len]);
                self.data.advance(now_len);
            } else {
                size += head.encode(dst)?;
                size += self.data.serialize(dst)?;
                break;
            }
        }
        Ok(size)
    }
}
