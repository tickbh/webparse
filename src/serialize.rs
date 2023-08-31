use std::{io::Write, borrow::Cow};

use crate::{WebResult, WebError, Buf, BufMut, MarkBuf, Binary, BinaryMut};

static EMPTY_ARRAY: Vec<u8> = vec![];
pub trait Serialize {
    fn serialize(&self, buffer: &mut BinaryMut) -> WebResult<()> {
        buffer.write(&self.serial_bytes()?).map_err(WebError::from)?;
        Ok(())
    }

    fn serialize_mut(&mut self, buffer: &mut BinaryMut) -> WebResult<()> {
        buffer.write(&self.serial_bytes()?).map_err(WebError::from)?;
        Ok(())
    }

    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>>;
}

pub trait Test {
    fn serialize<B: Buf + BufMut>(&self, buf: &mut B);
}

impl Test for &'static str {
    fn serialize<B: Buf + BufMut>(&self, buf: &mut B) {
        buf.put_slice(self.as_bytes());
    }
}

impl Serialize for &'static str {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(self.as_bytes()))
    }
}

impl Serialize for String {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(self.as_bytes()))
    }
}

impl Serialize for () {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(&EMPTY_ARRAY))
    }
}

impl Serialize for Vec<u8> {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(&self))
    }
}
