
use std::{io, num::Wrapping, sync::Arc};
use crate::{HeaderName, HeaderValue, Serialize};
use super::HeaderIndex;

pub struct Encoder {
    pub index: Arc<HeaderIndex>,
}


impl Encoder {
    
    pub fn new() -> Encoder {
        Encoder {
            index: Arc::new(HeaderIndex::new()),
        }
    }
    
    pub fn new_index(index: Arc<HeaderIndex>) -> Encoder {
        Encoder { index }
    }

    pub fn encode<'b, I>(&mut self, headers: I) -> Vec<u8>
            where I: Iterator<Item=(&'b HeaderName, &'b HeaderValue)> {
        let mut encoded: Vec<u8> = Vec::new();
        self.encode_into(headers, &mut encoded).unwrap();
        encoded
    }

    pub fn encode_into<'b, I, W>(&mut self, headers: I, writer: &mut W) -> io::Result<()>
            where I: Iterator<Item=(&'b HeaderName, &'b HeaderValue)>,
                  W: io::Write {
        for header in headers {
            self.encode_header_into(header, writer)?;
        }
        Ok(())
    }

    pub fn encode_header_into<W: io::Write>(
            &mut self,
            header: (&HeaderName, &HeaderValue),
            writer: &mut W)
            -> io::Result<()> {
        println!("header = {:?}", header);
        match self.index.find_header(header) {
            None => {
                self.encode_literal(header, true, writer)?;
                Arc::get_mut(&mut self.index).map(|v| {
                    v.add_header(header.0.clone(), header.1.clone());
                });
            },
            Some((index, false)) => {
                self.encode_indexed_name((index, &header.1), true, writer)?;
            },
            Some((index, true)) => {
                self.encode_indexed(index, writer)?;
            }
        };
        Ok(())
    }

    fn encode_literal<W: io::Write>(
        &mut self,
        header: (&HeaderName, &HeaderValue),
        should_index: bool,
        buf: &mut W)
        -> io::Result<()> {
        let mask = if should_index {
            0x40
        } else {
            0x0
        };

        buf.write_all(&[mask])?;
        self.encode_string_literal(&header.0.serial_bytes().unwrap(), buf)?;
        self.encode_string_literal(&header.1.serial_bytes().unwrap(), buf)?;
        Ok(())
    }

    fn encode_string_literal<W: io::Write>(
        &mut self,
        octet_str: &[u8],
        buf: &mut W)
        -> io::Result<()> {
        Self::encode_integer_into(octet_str.len(), 7, 0, buf)?;
        buf.write_all(octet_str)?;
        Ok(())
    }

    fn encode_indexed_name<W: io::Write>(
        &mut self,
        header: (usize, &HeaderValue),
        should_index: bool,
        buf: &mut W)
        -> io::Result<()> {
        let (mask, prefix) = if should_index {
            (0x40, 6)
        } else {
            (0x0, 4)
        };

        Self::encode_integer_into(header.0, prefix, mask, buf)?;
        // So far, we rely on just one strategy for encoding string literals.
        self.encode_string_literal(&header.1.serial_bytes().unwrap(), buf)?;
        Ok(())
    }

    fn encode_indexed<W: io::Write>(&self, index: usize, buf: &mut W) -> io::Result<()> {
        Self::encode_integer_into(index, 7, 0x80, buf)?;
        Ok(())
    }

    pub fn encode_integer_into<W: io::Write>(
        mut value: usize,
        prefix_size: u8,
        leading_bits: u8,
        writer: &mut W)
        -> io::Result<()> {
        let Wrapping(mask) = if prefix_size >= 8 {
            Wrapping(0xFF)
        } else {
            Wrapping(1u8 << prefix_size) - Wrapping(1)
        };
        let leading_bits = leading_bits & (!mask);
        let mask = mask as usize;
        if value < mask {
            writer.write_all(&[leading_bits | value as u8])?;
            return Ok(());
        }

        writer.write_all(&[leading_bits | mask as u8])?;
        value -= mask;
        while value >= 128 {
            writer.write_all(&[((value % 128) + 128) as u8])?;
            value = value / 128;
        }
        writer.write_all(&[value as u8])?;
        Ok(())
    }

}