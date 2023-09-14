use crate::{http::request, BufMut, HeaderName, Request, Serialize};
use std::fmt;

use crate::{
    http::{
        http2::{encoder::Encoder, Decoder},
        StatusCode,
    },
    Binary, BinaryMut, Buf, HeaderMap, Http2Error, MarkBuf, Method, Scheme, Url,
    WebResult,
};

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

#[derive(Debug)]
pub struct Continuation {
    /// Stream ID of continuation frame
    stream_id: StreamIdentifier,

    header_block: EncodingHeaderBlock,
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

#[derive(Debug)]
struct EncodingHeaderBlock {
    hpack: Binary,
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

    pub fn parse<B: Buf + MarkBuf>(
        &mut self,
        mut buffer: B,
        decoder: &mut Decoder,
        _max_header_list_size: usize,
    ) -> WebResult<usize> {
        let headers = decoder.decode(&mut buffer)?;
        for h in headers {
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
                    _ => {
                        self.header_block.fields.insert_exact(h.0, h.1);
                    }
                }
            } else {
                self.header_block.fields.insert_exact(h.0, h.1);
            }
        }
        Ok(buffer.mark_commit())
    }

    pub fn stream_id(&self) -> StreamIdentifier {
        self.stream_id
    }

    pub fn flags(&self) -> Flag {
        self.flags
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

    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.header_block.parts
    }

    /// Whether it has status 1xx
    pub(crate) fn is_informational(&self) -> bool {
        self.header_block.parts.is_informational()
    }

    pub fn fields(&self) -> &HeaderMap {
        &self.header_block.fields
    }

    pub fn into_fields(self) -> HeaderMap {
        self.header_block.fields
    }

    pub fn into_request(self, mut builder: request::Builder) -> WebResult<request::Builder> {
        let (parts, header) = self.into_parts();
        if let Some(m) = parts.method {
            builder = builder.method(m);
        }
        if let Some(path) = parts.path {
            let mut url = Url::parse(path.into_bytes())?;
            if let Some(authority) = parts.authority {
                url.domain = Some(authority);
            }
            builder = builder.url(url);
        }
        builder = builder.headers(header);
        Ok(builder)
    }

    pub fn encode<B: Buf + MarkBuf + BufMut>(mut self, encoder: &mut Encoder, dst: &mut B) -> WebResult<usize> {
        let _binary = BinaryMut::new();
        // let mut parts = self.header_block.parts;
        // let mut fields = self.header_block.fields;
        self.header_block.parts.encode_header(&mut self.header_block.fields);
        self.header_block.encode(encoder, dst, self.stream_id)

        // if let Some(status) = parts.status {
        //     // fields.insert(":status", status.as_str());
        //     let _ = encoder.encode_header_into(
        //         (
        //             &HeaderName::from_static(":status"),
        //             &HeaderValue::from_static(status.as_str()),
        //         ),
        //         &mut binary,
        //     );
        //     println!("stauts!!!!!!!!!");
        // } else {
        //     println!("other!!!!!!!!!");
        // }

        // for value in fields {
        //     if value.0.bytes_len() + value.1.bytes_len() + binary.remaining()
        //         > encoder.max_frame_size as usize
        //     {
        //         result.push(binary);
        //         binary = BinaryMut::new();
        //     }
        //     let _ = encoder.encode_header_into((&value.0, &value.1), &mut binary);
        // }

        // result.push(binary);
        // let mut size = 0;
        // if result.len() == 1 {
        //     let mut head = FrameHeader::new(Kind::Headers, self.flags.into(), self.stream_id);
        //     head.flag.set_end_headers();
        //     head.length = result[0].remaining() as u32;
        //     size += head.encode(dst).unwrap();
        //     size += result[0].serialize(dst).unwrap();
        // } else {
        //     let mut head = FrameHeader::new(Kind::Headers, self.flags.into(), self.stream_id);
        //     head.length = result[0].remaining() as u32;
        //     size += head.encode(dst).unwrap();
        //     size += result[0].serialize(dst).unwrap();

        //     for idx in 1..result.len() {
        //         let mut head =
        //             FrameHeader::new(Kind::Continuation, self.flags.into(), self.stream_id);
        //         if idx == result.len() - 1 {
        //             head.flag.set_end_headers();
        //         }
        //         head.length = result[idx].remaining() as u32;
        //         size += head.encode(dst).unwrap();
        //         size += result[idx].serialize(dst).unwrap();
        //     }
        // }
        // size
    }

    fn head(&self) -> FrameHeader {
        FrameHeader::new(Kind::Headers, self.flags.into(), self.stream_id)
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

    #[cfg(feature = "unstable")]
    pub fn into_fields(self) -> HeaderMap {
        self.header_block.fields
    }

    pub fn parse<B: Buf + MarkBuf>(
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

    pub fn encode<B: Buf + MarkBuf + BufMut>(mut self, encoder: &mut Encoder, dst: &mut B) -> WebResult<usize> {
        let mut binary = BinaryMut::new();
        self.header_block.parts.encode_header(&mut self.header_block.fields);

        if let Some(v) = self.header_block.fields.remove(":method") {
            let _ =
                encoder.encode_header_into((&HeaderName::from_static(":method"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(":authority") {
            let _ = encoder
                .encode_header_into((&HeaderName::from_static(":authority"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(":scheme") {
            let _ =
                encoder.encode_header_into((&HeaderName::from_static(":scheme"), &v), &mut binary);
        }
        if let Some(v) = self.header_block.fields.remove(":path") {
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

        self.header_block.encode(encoder, dst, self.promised_id)
        // binary = BinaryMut::new();

        // if let Some(status) = parts.status {
        //     // fields.insert(":status", status.as_str());
        //     let _ = encoder.encode_header_into(
        //         (
        //             &HeaderName::from_static(":status"),
        //             &HeaderValue::from_static(status.as_str()),
        //         ),
        //         &mut binary,
        //     );
        //     println!("stauts!!!!!!!!!");
        // } else {
        //     println!("other!!!!!!!!!");
        // }

        // let mut result = vec![];

        // for value in fields {
        //     if value.0.bytes_len() + value.1.bytes_len() + binary.remaining()
        //         > encoder.max_frame_size as usize
        //     {
        //         result.push(binary);
        //         binary = BinaryMut::new();
        //     }
        //     let _ = encoder.encode_header_into((&value.0, &value.1), &mut binary);
        // }

        // result.push(binary);
        // if result.len() == 1 {
        //     let mut head = FrameHeader::new(Kind::Headers, self.flags.into(), self.promised_id);
        //     head.flag.set_end_headers();
        //     head.length = result[0].remaining() as u32;
        //     size += head.encode(dst).unwrap();
        //     size += result[0].serialize(dst).unwrap();
        // } else {
        //     let mut head = FrameHeader::new(Kind::Headers, self.flags.into(), self.promised_id);
        //     head.length = result[0].remaining() as u32;
        //     size += head.encode(dst).unwrap();
        //     size += result[0].serialize(dst).unwrap();

        //     for idx in 1..result.len() {
        //         let mut head =
        //             FrameHeader::new(Kind::Continuation, self.flags.into(), self.stream_id);
        //         if idx == result.len() - 1 {
        //             head.flag.set_end_headers();
        //         }
        //         head.length = result[idx].remaining() as u32;
        //         size += head.encode(dst).unwrap();
        //         size += result[idx].serialize(dst).unwrap();
        //     }
        // }
        // size
    }

    fn head(&self) -> FrameHeader {
        FrameHeader::new(Kind::PushPromise, self.flags, self.stream_id)
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

// ===== impl Continuation =====

impl Continuation {
    fn head(&self) -> FrameHeader {
        FrameHeader::new(Kind::Continuation, Flag::end_headers(), self.stream_id)
    }

    pub fn parse<B: Buf>(self, _dst: &mut B) -> Option<Continuation> {
        // Get the CONTINUATION frame head
        let _head = self.head();
        // self.header_block.encode(&head, dst, |_| {})
        None
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

        // If the URI includes a scheme component, add it to the parts headers
        //
        // TODO: Scheme must be set...
        if uri.scheme != Scheme::None {
            parts.set_scheme(uri.scheme);
        }

        // If the URI includes an authority component, add it to the parts
        // headers
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

    #[cfg(feature = "unstable")]
    pub fn set_status(&mut self, value: StatusCode) {
        self.status = Some(value);
    }

    pub fn set_scheme(&mut self, scheme: Scheme) {
        self.scheme = Some(scheme);
    }

    #[cfg(feature = "unstable")]
    pub fn set_protocol(&mut self, protocol: Protocol) {
        self.protocol = Some(protocol);
    }

    pub fn set_authority(&mut self, authority: String) {
        self.authority = Some(authority);
    }

    /// Whether it has status 1xx
    pub(crate) fn is_informational(&self) -> bool {
        self.status
            .map_or(false, |status| status.is_informational())
    }

    pub fn encode_header(&mut self, header: &mut HeaderMap) {
        println!("fields = {:?}", header);
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
}

// ===== impl EncodingHeaderBlock =====

impl EncodingHeaderBlock {
    fn encode<F, B: Buf + MarkBuf + BufMut>(
        self,
        head: &mut FrameHeader,
        dst: &mut B,
        _f: F,
    ) -> Option<Continuation>
    where
        F: FnOnce(&mut BinaryMut),
    {
        let _head_pos = dst.remaining();

        // At this point, we don't know how big the h2 frame will be.
        // So, we write the head with length 0, then write the body, and
        // finally write the length once we know the size.
        head.encode(dst);

        let payload_pos = dst.remaining();

        // Now, encode the header payload
        let continuation = if self.hpack.len() > dst.remaining_mut() {
            // dst.put_slice(&self.hpack.split_to(dst.remaining_mut()));

            Some(Continuation {
                stream_id: head.stream_id(),
                header_block: self,
            })
        } else {
            dst.put_slice(&self.hpack);

            None
        };

        // Compute the header block length
        let _payload_len = (dst.remaining() - payload_pos) as u64;

        // Write the frame length
        // let payload_len_be = payload_len.to_be_bytes();
        // assert!(payload_len_be[0..5].iter().all(|b| *b == 0));
        // (dst.get_mut()[head_pos..head_pos + 3]).copy_from_slice(&payload_len_be[5..]);

        // if continuation.is_some() {
        //     // There will be continuation frames, so the `is_end_headers` flag
        //     // must be unset
        //     debug_assert!(dst[head_pos + 4] & END_HEADERS == END_HEADERS);

        //     dst.get_mut()[head_pos + 4] -= END_HEADERS;
        // }

        continuation
    }
}

// ===== impl Iter =====

// impl Iterator for Iter {
//     type Item = hpack::Header<Option<HeaderName>>;

//     fn next(&mut self) -> Option<Self::Item> {
//         use crate::hpack::Header::*;

//         if let Some(ref mut parts) = self.parts {
//             if let Some(method) = parts.method.take() {
//                 return Some(Method(method));
//             }

//             if let Some(scheme) = parts.scheme.take() {
//                 return Some(Scheme(scheme));
//             }

//             if let Some(authority) = parts.authority.take() {
//                 return Some(Authority(authority));
//             }

//             if let Some(path) = parts.path.take() {
//                 return Some(Path(path));
//             }

//             if let Some(protocol) = parts.protocol.take() {
//                 return Some(Protocol(protocol));
//             }

//             if let Some(status) = parts.status.take() {
//                 return Some(Status(status));
//             }
//         }

//         self.parts = None;

//         self.fields
//             .next()
//             .map(|(name, value)| Field { name, value })
//     }
// }

// ===== impl HeadersFlag =====

// ===== HeaderBlock =====

impl HeaderBlock {
    pub const FIRST: [&'static str; 5] = [":status", ":path", ":method", ":authority", ":scheme"];

    pub fn encode<B: Buf + BufMut + MarkBuf>(&mut self, encoder: &mut Encoder, dst: &mut B, stream_id: StreamIdentifier) -> WebResult<usize> {
        let mut result = vec![];
        let mut binary = BinaryMut::new();
        for key in Self::FIRST {
            if let Some(v) = self.fields.remove(key) {
                let _ = encoder.encode_header_into((&HeaderName::from_static(key), &v), &mut binary);
            }
        }
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
            let flag = Flag::end_headers();
            let mut head = FrameHeader::new(Kind::Headers, flag, stream_id);
            head.length = result[0].remaining() as u32;
            size += head.encode(dst).unwrap();
            size += result[0].serialize(dst).unwrap();
        } else {
            let mut head = FrameHeader::new(Kind::Headers, Flag::zero(), stream_id);
            head.length = result[0].remaining() as u32;
            size += head.encode(dst).unwrap();
            size += result[0].serialize(dst).unwrap();

            for idx in 1..result.len() {
                let mut head =
                    FrameHeader::new(Kind::Continuation, Flag::zero(), stream_id);
                if idx == result.len() - 1 {
                    head.flag.set_end_headers();
                }
                head.length = result[idx].remaining() as u32;
                size += head.encode(dst).unwrap();
                size += result[idx].serialize(dst).unwrap();
            }
        }
        Ok(size)
    }

    fn into_encoding(self, _encoder: &mut Encoder) -> EncodingHeaderBlock {
        let hpack = BinaryMut::new();
        // let headers = Iter {
        //     parts: Some(self.parts),
        //     fields: self.fields.into_iter(),
        // };

        // encoder.encode(headers, &mut hpack);

        EncodingHeaderBlock {
            hpack: hpack.freeze(),
        }
    }

    /// Calculates the size of the currently decoded header list.
    ///
    /// According to http://httpwg.org/specs/rfc7540.html#SETTINGS_MAX_HEADER_LIST_SIZE
    ///
    /// > The value is based on the uncompressed size of header fields,
    /// > including the length of the name and value in octets plus an
    /// > overhead of 32 octets for each header field.
    fn calculate_header_list_size(&self) -> usize {
        macro_rules! parts_size {
            ($name:ident) => {{
                self.parts
                    .$name
                    .as_ref()
                    .map(|m| decoded_header_size(stringify!($name).len() + 1, m.as_str().len()))
                    .unwrap_or(0)
            }};
        }
        0

        // parts_size!(method)
        //     + parts_size!(scheme)
        //     + parts_size!(status)
        //     + parts_size!(authority)
        //     + parts_size!(path)
        //     + self
        //         .fields
        //         .iter()
        //         .map(|(name, value)| decoded_header_size(name.as_str().len(), value.len()))
        //         .sum::<usize>()
    }
}

fn decoded_header_size(name: usize, value: usize) -> usize {
    name + value + 32
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
