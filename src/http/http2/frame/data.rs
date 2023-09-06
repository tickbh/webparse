
use crate::{Binary, Serialize, Buf, BufMut, MarkBuf};

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

    pub(crate) fn head(&self) -> FrameHeader {
        FrameHeader::new(Kind::Data, self.flags.into(), self.stream_id)
    }

    

    pub(crate) fn map<F, U>(self, f: F) -> Data<U>
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
    pub fn encode<B: Buf+MarkBuf+BufMut>(&self, dst: &mut B) -> usize {
        // Create & encode an appropriate frame head
        let mut head = FrameHeader::new(Kind::Data, self.flags.into(), self.stream_id);
        head.length = self.data.remaining() as u32;

        println!("encoding SETTINGS; len={}", head.length);
        let mut size = 0;
        size += head.serialize(dst).unwrap();
        size += self.data.serialize(dst).unwrap();
        size
    }
}

impl<T: Buf> Serialize for Data<T> {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> crate::WebResult<usize> {
        let mut size = 0;
        size += self.head().serialize(buffer)?;
        size += buffer.put_slice(self.data.chunk());
        Ok(size)
    }
}