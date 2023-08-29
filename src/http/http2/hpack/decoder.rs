use std::borrow::Cow;
use std::cell::RefCell;
use std::num::Wrapping;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::{WebResult, Buffer, HeaderName, HeaderValue, WebError, Http2Error, BinaryMut, Buf};

use super::huffman::{HuffmanDecoderError, HuffmanDecoder};
use super::HeaderIndex;


enum FieldRepresentation {
    Indexed,
    LiteralWithIncrementalIndexing,
    SizeUpdate,
    LiteralNeverIndexed,
    LiteralWithoutIndexing,
}

impl FieldRepresentation {
    fn new(octet: u8) -> FieldRepresentation {
        if octet & 128 == 128 {
            // High-order bit set
            FieldRepresentation::Indexed
        } else if octet & 64 == 64 {
            // Bit pattern `01`
            FieldRepresentation::LiteralWithIncrementalIndexing
        } else if octet & 32 == 32 {
            // Bit pattern `001`
            FieldRepresentation::SizeUpdate
        } else if octet & 16 == 16 {
            // Bit pattern `0001`
            FieldRepresentation::LiteralNeverIndexed
        } else {
            // None of the top 4 bits is set => bit pattern `0000xxxx`
            FieldRepresentation::LiteralWithoutIndexing
        }
    }
}


/// Represents all errors that can be encountered while decoding an
/// integer.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum IntegerDecodingError {
    /// 5.1. specifies that "excessively large integer decodings" MUST be
    /// considered an error (whether the size is the number of octets or
    /// value). This variant corresponds to the encoding containing too many
    /// octets.
    TooManyOctets,
    /// The variant corresponds to the case where the value of the integer
    /// being decoded exceeds a certain threshold.
    ValueTooLarge,
    /// When a buffer from which an integer was supposed to be encoded does
    /// not contain enough octets to complete the decoding.
    NotEnoughOctets,
    /// Only valid prefixes are [1, 8]
    InvalidPrefix,
}

/// Represents all errors that can be encountered while decoding an octet
/// string.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum StringDecodingError {
    NotEnoughOctets,
    HuffmanDecoderError(HuffmanDecoderError),
}

/// Represents all errors that can be encountered while performing the decoding
/// of an HPACK header set.
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Debug)]
pub enum DecoderError {
    HeaderIndexOutOfBounds,
    IntegerDecodingError(IntegerDecodingError),
    StringDecodingError(StringDecodingError),
    InvalidMaxDynamicSize,
}


pub struct Decoder {
    pub index: Arc<RwLock<HeaderIndex>>,
}

impl Decoder {

    pub fn new() -> Decoder {
        Decoder { index: Arc::new(RwLock::new(HeaderIndex::new())) }
    }

    pub fn new_index(index: Arc<RwLock<HeaderIndex>>) -> Decoder {
        Decoder { index }
    }

    pub fn decode(&mut self, buf: &mut BinaryMut) -> WebResult<Vec<(HeaderName, HeaderValue)>> {
        let mut header_list = Vec::new();
        self.decode_with_cb(buf, |n, v| header_list.push((n.into_owned(), v.into_owned())))?;
        Ok(header_list)
    }

    pub fn decode_with_cb<F>(&mut self, buf: &mut BinaryMut, mut cb: F) -> WebResult<()>
    where F: FnMut(Cow<HeaderName>, Cow<HeaderValue>) {
        while buf.has_remaining() {
            let initial_octet = buf.peek().unwrap();
            let buffer_leftover = buf.chunk();
            let consumed = match FieldRepresentation::new(initial_octet) {
                FieldRepresentation::Indexed => {
                    let consumed =
                        (self.decode_indexed(initial_octet, |name, value| {
                            cb(Cow::Borrowed(name), Cow::Borrowed(value));
                        }))?;
                    consumed
                },
                FieldRepresentation::LiteralWithIncrementalIndexing => {
                    let ((name, value), consumed) = {
                        let ((name, value), consumed) = 
                            self.decode_literal(buffer_leftover, true)?;
                        cb(Cow::Borrowed(&name), Cow::Borrowed(&value));

                        // Since we are to add the decoded header to the header table, we need to
                        // convert them into owned buffers that the decoder can keep internally.
                        let name = name.clone();
                        let value = value.clone();
                        ((name, value), consumed)
                    };
                    // // This cannot be done in the same scope as the `decode_literal` call, since
                    // // Rust cannot figure out that the `into_owned` calls effectively drop the
                    // // borrow on `self` that the `decode_literal` return value had. Since adding
                    // // a header to the table requires a `&mut self`, it fails to compile.
                    // // Manually separating it out here works around it...
                    self.index.write().unwrap().add_header(name, value);
                    consumed
                },
                FieldRepresentation::LiteralWithoutIndexing => {
                    // let ((name, value), consumed) =
                    //     try!(self.decode_literal(buffer_leftover, false));
                    // cb(name, value);

                    // consumed
                    0
                },
                FieldRepresentation::LiteralNeverIndexed => {
                    // // Same as the previous one, except if we were also a proxy
                    // // we would need to make sure not to change the
                    // // representation received here. We don't care about this
                    // // for now.
                    // let ((name, value), consumed) =
                    //     try!(self.decode_literal(buffer_leftover, false));
                    // cb(name, value);

                    // consumed
                    0
                },
                FieldRepresentation::SizeUpdate => {
                    // Handle the dynamic table size update...
                    // self.update_max_dynamic_size(buffer_leftover)
                    0
                }
            };

            buf.advance(consumed);
        }
        Ok(())
    }


    
    /// Decodes an integer encoded with a given prefix size (in bits).
    /// Assumes that the buffer `buf` contains the integer to be decoded,
    /// with the first byte representing the octet that contains the
    /// prefix.
    ///
    /// Returns a tuple representing the decoded integer and the number
    /// of bytes from the buffer that were used.
    fn decode_integer(buf: &[u8], prefix_size: u8)
        -> WebResult<(usize, usize)> {
            if prefix_size < 1 || prefix_size > 8 {
                return Err(Http2Error::into(DecoderError::IntegerDecodingError(
                    IntegerDecodingError::InvalidPrefix)));
            }
            if buf.len() < 1 {
                return Err(Http2Error::into(DecoderError::IntegerDecodingError(
                        IntegerDecodingError::NotEnoughOctets)));
            }

            // Make sure there's no overflow in the shift operation
            let Wrapping(mask) = if prefix_size == 8 {
                Wrapping(0xFF)
            } else {
                Wrapping(1u8 << prefix_size) - Wrapping(1)
            };
            let mut value = (buf[0] & mask) as usize;
            if value < (mask as usize) {
                // Value fits in the prefix bits.
                return Ok((value, 1));
            }

            let mut total = 1;
            let mut m = 0;
            let octet_limit = 5;

            for &b in buf[1..].iter() {
                total += 1;
                value += ((b & 127) as usize) * (1 << m);
                m += 7;

                if b & 128 != 128 {
                    // Most significant bit is not set => no more continuation bytes
                    return Ok((value, total));
                }

                if total == octet_limit {
                    // The spec tells us that we MUST treat situations where the
                    // encoded representation is too long (in octets) as an error.
                    return Err(Http2Error::into(DecoderError::IntegerDecodingError(
                            IntegerDecodingError::TooManyOctets)))
            }
        }

        // If we have reached here, it means the buffer has been exhausted without
        // hitting the termination condition.
        Err(Http2Error::into(DecoderError::IntegerDecodingError(
            IntegerDecodingError::NotEnoughOctets)))
    }

    fn decode_string<'a>(buf: &'a [u8]) -> WebResult<(Cow<'a, [u8]>, usize)> {
        let (len, consumed) = Self::decode_integer(buf, 7)?;
        // debug!("decode_string: Consumed = {}, len = {}", consumed, len);
        if consumed + len > buf.len() {
            return Err(Http2Error::into(
                DecoderError::StringDecodingError(
                    StringDecodingError::NotEnoughOctets)));
        }
        let raw_string = &buf[consumed..consumed + len];
        if buf[0] & 128 == 128 {
            // debug!("decode_string: Using the Huffman code");
            // Huffman coding used: pass the raw octets to the Huffman decoder
            // and return its result.
            let mut decoder = HuffmanDecoder::new();
            let decoded = match decoder.decode(raw_string) {
                Err(e) => {
                    return Err(e);
                },
                Ok(res) => res,
            };
            Ok((Cow::Owned(decoded), consumed + len))
        } else {
            // The octets were transmitted raw
            // debug!("decode_string: Raw octet string received");
            Ok((Cow::Borrowed(raw_string), consumed + len))
        }
    }

    fn decode_literal(&self, buf: &[u8], index: bool)
            -> WebResult<((HeaderName, HeaderValue), usize)> {
        let prefix = if index {
            6
        } else {
            4
        };
        let (table_index, mut consumed) = Self::decode_integer(buf, prefix)?;

        // First read the name appropriately
        let name = if table_index == 0 {
            // Read name string as literal
            let (name, name_len) = Self::decode_string(&buf[consumed..])?;
            consumed += name_len;
            HeaderName::from_bytes(&name).unwrap()
        } else {
            // Read name indexed from the table
            // let mut name;
            let mut name = HeaderName::Stand("");
            self.get_from_table(table_index, |n, _| {
                name = n.clone();
            })?;
            name
            // let (name, _) = self.get_from_table(table_index)?;
            // name.into_owned()
        };

        // Now read the value as a literal...
        let (value, value_len) = Self::decode_string(&buf[consumed..])?;
        consumed += value_len;

        Ok(((name, HeaderValue::from_bytes(&value)), consumed))
    }


    fn decode_indexed<F>(&self, index: u8, call: F) -> WebResult<usize> 
    where F : FnOnce(&HeaderName, &HeaderValue){
        let index = index & 0x7f;
        let header = self.index.read().unwrap();
        let (name, value) = header.get_from_index(index as usize).ok_or(Http2Error::into(DecoderError::HeaderIndexOutOfBounds))?;
        call(name, value);
        Ok(1)
    }

    fn get_from_table<F>(&self, index: usize, call: F) -> WebResult<()>
    where F : FnOnce(&HeaderName, &HeaderValue) {
        let header = self.index.read().unwrap();
        let (name, value) = header.get_from_index(index as usize).ok_or(Http2Error::into(DecoderError::HeaderIndexOutOfBounds))?;
        call(name, value);
        Ok(())
    }
}