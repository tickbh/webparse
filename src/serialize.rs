
use crate::{WebResult, Buf, BufMut, MarkBuf, Binary, BinaryMut};

static EMPTY_ARRAY: Vec<u8> = vec![];
pub trait Serialize {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize>;
}


impl Serialize for &'static str {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for String {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for () {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, _buffer: &mut B) -> WebResult<usize> {
        Ok(0)
    }
}

impl Serialize for Vec<u8> {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(&self))
    }
}

impl Serialize for Binary {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let len = self.remaining();
        buffer.put_slice(self.chunk());
        Ok(len)
    }
}

impl Serialize for BinaryMut {
    fn serialize<B: Buf+BufMut+MarkBuf>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let len = self.remaining();
        buffer.put_slice(self.chunk());
        Ok(len)
    }
}
