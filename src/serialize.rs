use std::io::Write;

use crate::{Buffer, WebResult, WebError};

pub trait Serialize {
    fn serialize(&self, buffer: &mut Buffer) -> WebResult<()>;
}

impl Serialize for String {
    fn serialize(&self, buffer: &mut Buffer) -> WebResult<()> {
        buffer.write(self.as_bytes()).map_err(WebError::from)?;
        Ok(())
    }
}