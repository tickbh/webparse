use std::{io::Write, borrow::Cow};

use crate::{WebResult, WebError, Buf, BufMut, MarkBuf, Binary, BinaryMut};

static EMPTY_ARRAY: Vec<u8> = vec![];
pub trait Serialize {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize>;
    // {
    //     Ok(buffer.put_slice(&self.serial_bytes()?))
    // }

    // fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>>;
}


impl Serialize for &'static str {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for String {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for () {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        Ok(0)
    }
}

impl Serialize for Vec<u8> {
    fn serialize<B: Buf+BufMut+MarkBuf>(&self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(&self))
    }
}
