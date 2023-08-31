
use std::{io, num::Wrapping, sync::{Arc, RwLock}, rc::Rc, cell::RefCell};
use crate::{HeaderName, HeaderValue, Serialize, BufMut, MarkBuf, Buf, BinaryMut};
use super::HeaderIndex;

pub struct Encoder {
    pub index: Arc<RwLock<HeaderIndex>>,
}


impl Encoder {

    pub fn new() -> Encoder {
        Encoder {
            index: Arc::new(RwLock::new(HeaderIndex::new())),
        }
    }
    
    pub fn new_index(index: Arc<RwLock<HeaderIndex>>) -> Encoder {
        Encoder { index }
    }

    pub fn encode<'b, I>(&mut self, headers: I) -> BinaryMut
            where I: Iterator<Item=(&'b HeaderName, &'b HeaderValue)> {
        let mut encoded = BinaryMut::new();
        self.encode_into(headers, &mut encoded).unwrap();
        encoded
    }

    pub fn encode_into<'b, I, B: BufMut + Buf + MarkBuf>(&mut self, headers: I, writer: &mut B) -> io::Result<()>
            where I: Iterator<Item=(&'b HeaderName, &'b HeaderValue)> {
        for header in headers {
            self.encode_header_into(header, writer)?;
        }
        Ok(())
    }

    pub fn encode_header_into<B: BufMut + Buf + MarkBuf>(
            &mut self,
            header: (&HeaderName, &HeaderValue),
            writer: &mut B)
            -> io::Result<()> {
        println!("header = {:?}", header);
        let value = {
            self.index.read().unwrap().find_header(header)
        };
        match value {
            None => {
                self.encode_literal(header, true, writer)?;
                self.index.write().unwrap().add_header(header.0.clone(), header.1.clone());

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

    fn encode_literal<B: BufMut + Buf + MarkBuf>(
        &mut self,
        header: (&HeaderName, &HeaderValue),
        should_index: bool,
        buf: &mut B)
        -> io::Result<()> {
        let mask = if should_index {
            0x40
        } else {
            0x0
        };

        buf.put_slice(&[mask]);
        self.encode_string_literal(&header.0.as_bytes(), buf)?;
        self.encode_string_literal(&header.1.as_bytes(), buf)?;
        Ok(())
    }

    fn encode_string_literal<B: BufMut + Buf + MarkBuf>(
        &mut self,
        octet_str: &[u8],
        buf: &mut B)
        -> io::Result<()> {
        Self::encode_integer_into(octet_str.len(), 7, 0, buf)?;
        buf.put_slice(octet_str);
        Ok(())
    }

    fn encode_indexed_name<B: BufMut + Buf + MarkBuf>(
        &mut self,
        header: (usize, &HeaderValue),
        should_index: bool,
        buf: &mut B)
        -> io::Result<()> {
        let (mask, prefix) = if should_index {
            (0x40, 6)
        } else {
            (0x0, 4)
        };

        Self::encode_integer_into(header.0, prefix, mask, buf)?;
        // So far, we rely on just one strategy for encoding string literals.
        self.encode_string_literal(&header.1.as_bytes(), buf)?;
        Ok(())
    }

    fn encode_indexed<B: BufMut + Buf + MarkBuf>(&self, index: usize, buf: &mut B) -> io::Result<()> {
        Self::encode_integer_into(index, 7, 0x80, buf)?;
        Ok(())
    }

    pub fn encode_integer_into<B: BufMut + Buf + MarkBuf>(
        mut value: usize,
        prefix_size: u8,
        leading_bits: u8,
        writer: &mut B)
        -> io::Result<()> {
        let Wrapping(mask) = if prefix_size >= 8 {
            Wrapping(0xFF)
        } else {
            Wrapping(1u8 << prefix_size) - Wrapping(1)
        };
        let leading_bits = leading_bits & (!mask);
        let mask = mask as usize;
        if value < mask {
            writer.put_slice(&[leading_bits | value as u8]);
            return Ok(());
        }

        writer.put_slice(&[leading_bits | mask as u8]);
        value -= mask;
        while value >= 128 {
            writer.put_slice(&[((value % 128) + 128) as u8]);
            value = value / 128;
        }
        writer.put_slice(&[value as u8]);
        Ok(())
    }

}