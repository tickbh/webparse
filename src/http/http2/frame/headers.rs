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
// Created Date: 2023/09/01 04:34:25

use crate::{
    http::{request, response},
    http2::DecoderError,
    HeaderName, Request, Serialize,
};
use std::fmt;

use crate::{
    http::{
        http2::{encoder::Encoder, Decoder},
        StatusCode,
    },
    HeaderMap, Http2Error, Method, Scheme, Url, WebResult,
};
use algorithm::buf::{BinaryMut, Bt, BtMut};

use super::{frame::Frame, Flag, FrameHeader, Kind, StreamDependency, StreamIdentifier};

///
/// This could be either a request or a response.
#[derive(Eq, PartialEq)]
pub struct Headers {
    /// The ID of the stream with which this frame is associated.
    stream_id: StreamIdentifier,

    /// The stream dependency information, if any.
    stream_dep: Option<StreamDependency>,

    /// The header block fragment
    header_block: HeaderBlock,

    /// The associated flags
    flags: Flag,
}

#[derive(Eq, PartialEq)]
pub struct PushPromise {
    /// The ID of the stream with which this frame is associated.
    stream_id: StreamIdentifier,

    /// The ID of the stream being reserved by this PushPromise.
    promised_id: StreamIdentifier,

    /// The header block fragment
    header_block: HeaderBlock,

    /// The associated flags
    flags: Flag,
}

// TODO: These fields shouldn't be `pub`
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Parts {
    // Request
    pub method: Option<Method>,
    pub scheme: Option<Scheme>,
    pub authority: Option<String>,
    pub path: Option<String>,

    // Response
    pub status: Option<StatusCode>,
}

#[derive(Debug, PartialEq, Eq)]
struct HeaderBlock {
    /// 解析的头列表
    fields: HeaderMap,

    /// 超出头列表的限制则为true
    is_over_size: bool,

    /// 保存部分的头文件信息, 如Method等做完转换的
    parts: Parts,
}

impl Headers {
    /// Create a new HEADERS frame
    pub fn trailers(stream_id: StreamIdentifier, parts: Parts, fields: HeaderMap) -> Self {
        Headers {
            stream_id,
            stream_dep: None,
            header_block: HeaderBlock {
                fields,
                is_over_size: false,
                parts,
            },
            flags: Flag::default(),
        }
    }

    pub fn new(header: FrameHeader, fields: HeaderMap) -> Self {
        Headers {
            stream_id: header.stream_id(),
            stream_dep: None,
            header_block: HeaderBlock {
                fields,
                is_over_size: false,
                parts: Parts::default(),
            },
            flags: header.flag(),
        }
    }

    pub fn empty() -> Self {
        Headers {
            stream_id: StreamIdentifier::zero(),
            stream_dep: None,
            header_block: HeaderBlock {
                fields: HeaderMap::new(),
                is_over_size: false,
                parts: Parts::default(),
            },
            flags: Flag::zero(),
        }
    }

    pub fn parse<B: Bt>(
        &mut self,
        mut buffer: B,
        decoder: &mut Decoder,
        max_header_list_size: usize,
    ) -> WebResult<usize> {
        if self.flags.is_priority() {
            let depency = StreamDependency::load(&mut buffer)?;
            self.stream_dep = Some(depency);
        }

        let len = buffer.remaining();
        let headers = decoder.decode(&mut buffer)?;
        let mut header_size = 0;
        for h in headers {
            header_size += h.0.as_bytes().len() + h.1.as_bytes().len() + 32;
            if header_size > max_header_list_size {
                return Err(Http2Error::Decoder(DecoderError::HeaderIndexOutOfBounds).into());
            }
            if h.0.is_spec() {
                let value: String = (&h.1).try_into()?;
                match h.0.name() {
                    ":authority" => {
                        self.header_block.parts.authority = Some(value);
                    }
                    ":method" => {
                        self.header_block.parts.method = Some(Method::try_from(&*value)?);
                    }
                    ":path" => {
                        self.header_block.parts.path = Some(value);
                    }
                    ":scheme" => {
                        self.header_block.parts.scheme = Some(Scheme::try_from(&*value)?);
                    }
                    ":status" => {
                        self.header_block.parts.status = Some(StatusCode::try_from(&*value)?);
                    }
                    _ => {
                        self.header_block.fields.insert(h.0, h.1);
                    }
                }
            } else {
                self.header_block.fields.insert(h.0, h.1);
            }
        }
        Ok(len - buffer.remaining())
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn flags(&self) -> Flag {
        self.flags
    }

    pub fn flags_mut(&mut self) -> &mut Flag {
        &mut self.flags
    }

    pub fn is_end_headers(&self) -> bool {
        self.flags.is_end_headers()
    }

    pub fn set_end_headers(&mut self) {
        self.flags.set_end_headers();
    }

    pub fn is_end_stream(&self) -> bool {
        self.flags.is_end_stream()
    }

    pub fn set_end_stream(&mut self) {
        self.flags.set_end_stream()
    }

    pub fn set_method(&mut self, method: Method) {
        self.header_block.parts.method = Some(method);
    }

    pub fn method(&mut self) -> &Option<Method> {
        &self.header_block.parts.method
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.header_block.parts.scheme = Some(scheme);
    }

    pub fn scheme(&mut self) -> &Option<Scheme> {
        &self.header_block.parts.scheme
    }

    pub fn set_authority(&mut self, authority: String) {
        self.header_block.parts.authority = Some(authority);
    }

    pub fn authority(&mut self) -> &Option<String> {
        &self.header_block.parts.authority
    }

    pub fn set_path(&mut self, path: String) {
        self.header_block.parts.path = Some(path);
    }

    pub fn path(&mut self) -> &Option<String> {
        &self.header_block.parts.path
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.header_block.parts.status = Some(status);
    }

    pub fn status(&mut self) -> &Option<StatusCode> {
        &self.header_block.parts.status
    }

    pub fn is_over_size(&self) -> bool {
        self.header_block.is_over_size
    }

    pub fn into_parts(self) -> (Parts, HeaderMap) {
        (self.header_block.parts, self.header_block.fields)
    }

    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.header_block.parts
    }

    /// Whether it has status 1xx
    pub fn is_informational(&self) -> bool {
        self.header_block.parts.is_informational()
    }

    pub fn fields_mut(&mut self) -> &mut HeaderMap {
        &mut self.header_block.fields
    }

    pub fn fields(&self) -> &HeaderMap {
        &self.header_block.fields
    }

    pub fn into_fields(self) -> HeaderMap {
        self.header_block.fields
    }

    pub fn into_request(self, mut builder: request::Builder) -> WebResult<request::Builder> {
        let (parts, header) = self.into_parts();
        let url = parts.build_url()?;
        builder = builder.url(url);
        if let Some(m) = parts.method {
            builder = builder.method(m);
        }
        builder = builder.headers(header);
        Ok(builder)
    }

    pub fn into_response(self, mut builder: response::Builder) -> WebResult<response::Builder> {
        let (parts, header) = self.into_parts();
        if let Some(m) = parts.method {
            builder = builder.method(m.as_str().to_string());
        }
        if let Some(status) = parts.status {
            builder = builder.status(status);
        }
        builder = builder.headers(header);
        Ok(builder)
    }

    pub fn encode<B: Bt + BtMut>(mut self, encoder: &mut Encoder, dst: &mut B) -> WebResult<usize> {
        let size = self
            .header_block
            .encode(encoder, dst, self.flags, self.stream_id)?;
        log::trace!("HTTP2: 编码头信息; len={}", size);
        Ok(size)
    }
}

impl<T> From<Headers> for Frame<T> {
    fn from(src: Headers) -> Self {
        Frame::Headers(src)
    }
}

impl fmt::Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("Headers");
        builder
            .field("stream_id", &self.stream_id)
            .field("flags", &self.flags);

        if let Some(ref dep) = self.stream_dep {
            builder.field("stream_dep", dep);
        }

        // `fields` and `parts` purposefully not included
        builder.finish()
    }
}

// ===== impl PushPromise =====

impl PushPromise {
    pub fn new(header: FrameHeader, promised_id: StreamIdentifier, fields: HeaderMap) -> Self {
        PushPromise {
            flags: header.flag(),
            header_block: HeaderBlock {
                fields,
                is_over_size: false,
                parts: Parts::default(),
            },
            promised_id,
            stream_id: header.stream_id(),
        }
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn flags(&self) -> Flag {
        self.flags
    }

    pub fn flags_mut(&mut self) -> &mut Flag {
        &mut self.flags
    }

    pub fn is_end_headers(&self) -> bool {
        self.flags.is_end_headers()
    }

    pub fn set_end_headers(&mut self) {
        self.flags.set_end_headers();
    }

    pub fn is_end_stream(&self) -> bool {
        self.flags.is_end_stream()
    }

    pub fn set_end_stream(&mut self) {
        self.flags.set_end_stream()
    }

    pub fn set_method(&mut self, method: Method) {
        self.header_block.parts.method = Some(method);
    }

    pub fn method(&mut self) -> &Option<Method> {
        &self.header_block.parts.method
    }

    pub fn set_authority(&mut self, authority: String) {
        self.header_block.parts.authority = Some(authority);
    }

    pub fn authority(&mut self) -> &Option<String> {
        &self.header_block.parts.authority
    }

    pub fn set_path(&mut self, path: String) {
        self.header_block.parts.path = Some(path);
    }

    pub fn path(&mut self) -> &Option<String> {
        &self.header_block.parts.path
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.header_block.parts.status = Some(status);
    }

    pub fn status(&mut self) -> &Option<StatusCode> {
        &self.header_block.parts.status
    }

    pub fn is_over_size(&self) -> bool {
        self.header_block.is_over_size
    }

    pub fn into_parts(self) -> (Parts, HeaderMap) {
        (self.header_block.parts, self.header_block.fields)
    }

    pub fn validate_request(req: &Request<()>) -> WebResult<()> {
        // use PushPromiseHeaderHttp2Error::*;
        // The spec has some requirements for promised request headers
        // [https://httpwg.org/specs/rfc7540.html#PushRequests]

        if req.get_body_len() == 0 {
            return Err(Http2Error::PayloadLengthTooShort.into());
        }
        // "The server MUST include a method in the :method parts-header field
        // that is safe and cacheable"
        if !Self::safe_and_cacheable(req.method()) {
            // return Err(NotSafeAndCacheable);
            return Err(Http2Error::PayloadLengthTooShort.into());
        }

        Ok(())
    }

    fn safe_and_cacheable(method: &Method) -> bool {
        // Cacheable: https://httpwg.org/specs/rfc7231.html#cacheable.methods
        // Safe: https://httpwg.org/specs/rfc7231.html#safe.methods
        method == &Method::GET || method == &Method::HEAD
    }

    pub fn fields(&self) -> &HeaderMap {
        &self.header_block.fields
    }

    pub fn into_fields(self) -> HeaderMap {
        self.header_block.fields
    }

    pub fn parse<B: Bt>(
        head: FrameHeader,
        mut src: B,
        _decoder: &mut Decoder,
        _max_header_list_size: usize,
    ) -> WebResult<Self> {
        let promised_id = StreamIdentifier::parse(&mut src);
        let push = PushPromise::new(head, promised_id, HeaderMap::new());
        // push.header_block
        //     .parse(&mut src, max_header_list_size, decoder)?;
        Ok(push)
    }

    pub fn promised_id(&self) -> StreamIdentifier {
        self.promised_id
    }

    pub fn encode<B: Bt + BtMut>(mut self, encoder: &mut Encoder, dst: &mut B) -> WebResult<usize> {
        let mut binary = BinaryMut::new();
        self.header_block
            .parts
            .encode_header(&mut self.header_block.fields);

        if let Some(v) = self.header_block.fields.remove(&":method") {
            let _ =
                encoder.encode_header_into((&HeaderName::from_static(":method"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(&":authority") {
            let _ = encoder
                .encode_header_into((&HeaderName::from_static(":authority"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(&":scheme") {
            let _ =
                encoder.encode_header_into((&HeaderName::from_static(":scheme"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(&":path") {
            let _ =
                encoder.encode_header_into((&HeaderName::from_static(":path"), &v), &mut binary);
        }

        let mut size = 0;
        let mut head = FrameHeader::new(Kind::PushPromise, self.flags.into(), self.stream_id);
        head.flag.set_end_headers();
        head.length = binary.remaining() as u32 + 4;
        size += head.encode(dst).unwrap();
        size += self.promised_id.encode(dst).unwrap();
        size += binary.serialize(dst).unwrap();

        size += self
            .header_block
            .encode(encoder, dst, self.flags, self.promised_id)?;
        log::trace!("HTTP2: 编码推送信息; len={}", size);
        Ok(size)
    }
}

impl<T> From<PushPromise> for Frame<T> {
    fn from(src: PushPromise) -> Self {
        Frame::PushPromise(src)
    }
}

impl fmt::Debug for PushPromise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PushPromise")
            .field("stream_id", &self.stream_id)
            .field("promised_id", &self.promised_id)
            .field("flags", &self.flags)
            // `fields` and `parts` purposefully not included
            .finish()
    }
}

// ===== impl parts =====

impl Parts {
    pub fn request(method: Method, uri: Url, protocol: Option<Scheme>) -> Self {
        let path = uri.path;

        let mut parts = Parts {
            method: Some(method),
            scheme: protocol,
            authority: None,
            path: Some(path).filter(|p| !p.is_empty()),
            status: None,
        };

        if uri.scheme != Scheme::None {
            parts.set_scheme(uri.scheme);
        }

        if let Some(authority) = uri.domain {
            parts.set_authority(authority);
        }

        parts
    }

    pub fn response(status: StatusCode) -> Self {
        Parts {
            method: None,
            scheme: None,
            authority: None,
            path: None,
            status: Some(status),
        }
    }

    pub fn set_status(&mut self, value: StatusCode) {
        self.status = Some(value);
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.scheme = Some(scheme);
    }

    pub fn set_authority(&mut self, authority: String) {
        self.authority = Some(authority);
    }

    pub fn is_informational(&self) -> bool {
        self.status
            .map_or(false, |status| status.is_informational())
    }

    pub fn encode_header(&mut self, header: &mut HeaderMap) {
        if let Some(method) = self.method.take() {
            header.insert(":method", method.as_str().to_string());
        }
        if let Some(authority) = self.authority.take() {
            header.insert(":authority", authority);
        }
        if let Some(scheme) = self.scheme.take() {
            header.insert(":scheme", scheme.as_str().to_string());
        }
        if let Some(path) = self.path.take() {
            header.insert(":path", path);
        }
        if let Some(status) = self.status.take() {
            header.insert(":status", status.as_str());
        }
    }

    pub fn build_url(&self) -> WebResult<Url> {
        if self.authority.is_none() {
            return Err(crate::WebError::Http2(Http2Error::InvalidRequesetUrl));
        }
        let url = format!(
            "{}://{}{}",
            self.scheme.as_ref().unwrap_or(&Scheme::Http),
            self.authority.as_ref().unwrap(),
            self.path.clone().unwrap_or("/".to_string())
        );
        let url = Url::parse(url.into_bytes().to_vec())?;
        Ok(url)
    }
}

impl HeaderBlock {
    pub fn encode<B: Bt + BtMut>(
        &mut self,
        encoder: &mut Encoder,
        dst: &mut B,
        mut flags: Flag,
        stream_id: StreamIdentifier,
    ) -> WebResult<usize> {
        let mut result = vec![];
        let mut binary = BinaryMut::new();

        for value in self.fields.iter() {
            if value.0.bytes_len() + value.1.bytes_len() + binary.remaining()
                > encoder.max_frame_size as usize
            {
                result.push(binary);
                binary = BinaryMut::new();
            }
            let _ = encoder.encode_header_into((&value.0, &value.1), &mut binary);
        }

        result.push(binary);
        let mut size = 0;
        if result.len() == 1 {
            flags.set_end_headers();
            let mut head = FrameHeader::new(Kind::Headers, flags, stream_id);
            head.length = result[0].remaining() as u32;
            size += head.encode(dst).unwrap();
            size += result[0].serialize(dst).unwrap();
        } else {
            let mut head = FrameHeader::new(Kind::Headers, Flag::zero(), stream_id);
            head.length = result[0].remaining() as u32;
            size += head.encode(dst).unwrap();
            size += result[0].serialize(dst).unwrap();

            for idx in 1..result.len() {
                let mut head = FrameHeader::new(Kind::Continuation, Flag::zero(), stream_id);
                if idx == result.len() - 1 {
                    flags.set_end_headers();
                    head.flag = flags;
                }
                head.length = result[idx].remaining() as u32;
                size += head.encode(dst).unwrap();
                size += result[idx].serialize(dst).unwrap();
            }
        }
        Ok(size)
    }
}

// #[cfg(test)]
// mod test {
//     use std::iter::FromIterator;

//     use http::HeaderValue;

//     use super::*;
//     use crate::{frame, BinaryMut, HeaderName};
//     use crate::hpack::{huffman, Encoder};

//     #[test]
//     fn test_nameless_header_at_resume() {
//         let mut encoder = Encoder::default();
//         let mut dst = BinaryMut::new();

//         let headers = Headers::new(
//             StreamIdentifier::zero(),
//             Default::default(),
//             HeaderMap::from_iter(vec![
//                 (
//                     HeaderName::from_static("hello"),
//                     HeaderValue::from_static("world"),
//                 ),
//                 (
//                     HeaderName::from_static("hello"),
//                     HeaderValue::from_static("zomg"),
//                 ),
//                 (
//                     HeaderName::from_static("hello"),
//                     HeaderValue::from_static("sup"),
//                 ),
//             ]),
//         );

//         let continuation = headers
//             .encode(&mut encoder, &mut (&mut dst).limit(frame::HEADER_LEN + 8))
//             .unwrap();

//         assert_eq!(17, dst.len());
//         assert_eq!([0, 0, 8, 1, 0, 0, 0, 0, 0], &dst[0..9]);
//         assert_eq!(&[0x40, 0x80 | 4], &dst[9..11]);
//         assert_eq!("hello", huff_decode(&dst[11..15]));
//         assert_eq!(0x80 | 4, dst[15]);

//         let mut world = dst[16..17].to_owned();

//         dst.clear();

//         assert!(continuation
//             .encode(&mut (&mut dst).limit(frame::HEADER_LEN + 16))
//             .is_none());

//         world.extend_from_slice(&dst[9..12]);
//         assert_eq!("world", huff_decode(&world));

//         assert_eq!(24, dst.len());
//         assert_eq!([0, 0, 15, 9, 4, 0, 0, 0, 0], &dst[0..9]);

//         // // Next is not indexed
//         assert_eq!(&[15, 47, 0x80 | 3], &dst[12..15]);
//         assert_eq!("zomg", huff_decode(&dst[15..18]));
//         assert_eq!(&[15, 47, 0x80 | 3], &dst[18..21]);
//         assert_eq!("sup", huff_decode(&dst[21..]));
//     }

//     fn huff_decode(src: &[u8]) -> BinaryMut {
//         let mut buf = BinaryMut::new();
//         huffman::decode(src, &mut buf).unwrap()
//     }
// }
