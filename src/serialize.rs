use std::{io::Write, borrow::Cow};

use crate::{Buffer, WebResult, WebError};

pub trait Serialize {
    fn serialize(&self, buffer: &mut Buffer) -> WebResult<()> {
        buffer.write(&self.serial_bytes()?).map_err(WebError::from)?;
        Ok(())
    }

    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>>;
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
        Err(WebError::Serialize("() can't serial bytes"))
    }
}

impl Serialize for Vec<u8> {
    fn serial_bytes<'a>(&'a self) -> WebResult<Cow<'a, [u8]>> {
        Ok(Cow::Borrowed(&self))
    }
}
