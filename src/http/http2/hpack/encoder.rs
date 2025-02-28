// Copyright 2022 - 2023 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Author: tickbh
// -----
// Created Date: 2023/08/22 10:50:59

use super::{huffman::HuffmanEncoder, HeaderIndex};
use crate::{HeaderName, HeaderValue};
use algorithm::buf::{BinaryMut, Bt, BtMut};
use std::{
    io,
    num::Wrapping,
    sync::{Arc, RwLock},
};

pub struct Encoder {
    pub index: Arc<RwLock<HeaderIndex>>,
    pub max_frame_size: usize,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder {
            index: Arc::new(RwLock::new(HeaderIndex::new())),
            max_frame_size: 16_384,
        }
    }

    pub fn new_index(index: Arc<RwLock<HeaderIndex>>, max_frame_size: usize) -> Encoder {
        Encoder {
            index,
            max_frame_size,
        }
    }

    pub fn encode<'b, I>(&mut self, headers: I) -> BinaryMut
    where
        I: Iterator<Item = (&'b HeaderName, &'b HeaderValue)>,
    {
        let mut encoded = BinaryMut::new();
        self.encode_into(headers, &mut encoded).unwrap();
        encoded
    }

    pub fn encode_into<'b, I, B: BtMut + Bt>(
        &mut self,
        headers: I,
        writer: &mut B,
    ) -> io::Result<()>
    where
        I: Iterator<Item = (&'b HeaderName, &'b HeaderValue)>,
    {
        for header in headers {
            self.encode_header_into(header, writer)?;
        }
        Ok(())
    }

    pub fn encode_header_into<B: BtMut + Bt>(
        &mut self,
        header: (&HeaderName, &HeaderValue),
        writer: &mut B,
    ) -> io::Result<()> {
        let value = { self.index.read().unwrap().find_header(header) };

        match value {
            None => {
                self.encode_literal(header, true, writer)?;
                self.index
                    .write()
                    .unwrap()
                    .add_header(header.0.clone(), header.1.clone());
            }
            Some((index, false)) => {
                self.encode_indexed_name((index, &header.1), true, writer)?;
                self.index
                    .write()
                    .unwrap()
                    .add_header(header.0.clone(), header.1.clone());
            }
            Some((index, true)) => {
                self.encode_indexed(index, writer)?;
            }
        };
        Ok(())
    }

    fn encode_literal<B: BtMut + Bt>(
        &mut self,
        header: (&HeaderName, &HeaderValue),
        should_index: bool,
        buf: &mut B,
    ) -> io::Result<()> {
        let mask = if should_index { 0x40 } else { 0x0 };

        buf.put_slice(&[mask]);
        self.encode_string_literal_lower(&header.0.as_bytes(), buf)?;
        self.encode_string_literal(&header.1.as_bytes(), buf)?;
        Ok(())
    }

    fn encode_string_literal_lower<B: BtMut + Bt>(
        &mut self,
        octet_str: &[u8],
        buf: &mut B,
    ) -> io::Result<()> {
        let value = HuffmanEncoder::encode_lower(octet_str);
        Self::encode_integer_into(value.len(), 7, 0x80, buf)?;
        buf.put_slice(&value);
        Ok(())
    }

    fn encode_string_literal<B: BtMut + Bt>(
        &mut self,
        octet_str: &[u8],
        buf: &mut B,
    ) -> io::Result<()> {
        let value = HuffmanEncoder::encode(octet_str);
        Self::encode_integer_into(value.len(), 7, 0x80, buf)?;
        buf.put_slice(&value);
        Ok(())
    }

    fn encode_indexed_name<B: BtMut + Bt>(
        &mut self,
        header: (usize, &HeaderValue),
        should_index: bool,
        buf: &mut B,
    ) -> io::Result<()> {
        let (mask, prefix) = if should_index { (0x40, 6) } else { (0x0, 4) };

        Self::encode_integer_into(header.0, prefix, mask, buf)?;
        // So far, we rely on just one strategy for encoding string literals.
        self.encode_string_literal(&header.1.as_bytes(), buf)?;
        Ok(())
    }

    fn encode_indexed<B: BtMut + Bt>(&self, index: usize, buf: &mut B) -> io::Result<()> {
        Self::encode_integer_into(index, 7, 0x80, buf)?;
        Ok(())
    }

    pub fn encode_integer_into<B: BtMut + Bt>(
        mut value: usize,
        prefix_size: u8,
        leading_bits: u8,
        writer: &mut B,
    ) -> io::Result<()> {
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
