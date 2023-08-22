
use std::io;

use crate::{HeaderName, HeaderValue};

// use super::
// use super::
use super::HeaderIndex;

pub struct Encoder {
    pub index: HeaderIndex,
}


impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            index: HeaderIndex::new(),
        }
    }

    pub fn encode<'b, I>(&mut self, headers: I) -> Vec<u8>
            where I: IntoIterator<Item=&'b (HeaderName, HeaderValue)> {
        let mut encoded: Vec<u8> = Vec::new();
        self.encode_into(headers, &mut encoded).unwrap();
        encoded
    }

    pub fn encode_into<'b, I, W>(&mut self, headers: I, writer: &mut W) -> io::Result<()>
            where I: IntoIterator<Item=&'b (HeaderName, HeaderValue)>,
                  W: io::Write {
        for header in headers {
            self.encode_header_into(header, writer)?;
        }
        Ok(())
    }

    pub fn encode_header_into<W: io::Write>(
            &mut self,
            header: &(HeaderName, HeaderValue),
            writer: &mut W)
            -> io::Result<()> {
        match self.index.find_header(header) {
            None => {
                // The name of the header is in no tables: need to encode
                // it with both a literal name and value.
                // self.encode_literal(&header, true, writer)?;
                // self.header_table.add_header(header.0.to_vec(), header.1.to_vec());
            },
            Some((index, false)) => {
                // The name of the header is at the given index, but the
                // value does not match the current one: need to encode
                // only the value as a literal.
                // self.encode_indexed_name((index, header.1), false, writer)?;
            },
            Some((index, true)) => {
                // The full header was found in one of the tables, so we
                // just encode the index.
                // self.encode_indexed(index, writer)?;
            }
        };
        Ok(())
    }

}