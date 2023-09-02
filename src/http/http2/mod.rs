pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

use std::{borrow::Cow, fmt::Debug};
mod error;
mod frame;
mod hpack;

pub use error::Http2Error;

use crate::{
    http::http2::frame::Settings, serialize, Binary, BinaryMut, Buf, BufMut, MarkBuf, Method,
    Request, Response, Serialize, WebError, WebResult,
};
pub use hpack::*;

use self::frame::{Flag, Frame, FrameHeader, Kind, StreamIdentifier};

pub struct Http2;

impl Http2 {
    pub fn parse_buffer<T: serialize::Serialize, B: Buf + MarkBuf>(
        request: &mut Request<T>,
        buffer: &mut B,
    ) -> WebResult<()> {
        // while buffer.has_remaining() {
        //     let frame_header = FrameHeader::parse(buffer)?;
        //     let frame = Frame::parse(frame_header, buffer)?;
        //     println!("frame = {:?}", frame);
        //     match frame.payload {
        //         Payload::Data { data } => {}
        //         Payload::Headers {
        //             priority,
        //             mut block,
        //         } => {
        //             request.parse_http2_header(&mut block)?;
        //             if request.method().is_nobody() {
        //                 return Ok(());
        //             }
        //         }
        //         Payload::Priority(priority) => {}
        //         Payload::Reset(err) => {}
        //         Payload::Settings(s) => {
        //             let frame = Frame {
        //                 header: FrameHeader {
        //                     length: 0,
        //                     kind: Kind::Settings,
        //                     flag: Flag::ack(),
        //                     id: StreamIdentifier(0),
        //                 },
        //                 payload: Payload::Settings::<Binary>(Settings::default()),
        //             };
        //             let has = {
        //                 request
        //                     .extensions()
        //                     .borrow()
        //                     .get::<Vec<Frame<Binary>>>()
        //                     .is_some()
        //             };
        //             if has {
        //                 request
        //                     .extensions()
        //                     .borrow_mut()
        //                     .get_mut::<Vec<Frame<Binary>>>()
        //                     .unwrap()
        //                     .push(frame);
        //             } else {
        //                 let vec = vec![frame];
        //                 request.extensions().borrow_mut().insert(vec);
        //             }
        //         }
        //         Payload::WindowUpdate(s) => {}
        //         _ => {}
        //     }
        // }
        Ok(())
    }

    // pub fn build_body_frame<T: serialize::Serialize>(
    //     res: &mut Response<T>,
    // ) -> WebResult<Option<Frame<BinaryMut>>> {
    //     let mut buf = BinaryMut::new();
    //     res.body().serialize(&mut buf)?;
    //     if buf.remaining() == 0 {
    //         return Ok(None);
    //     }
    //     let header = FrameHeader {
    //         length: buf.remaining() as u32,
    //         kind: Kind::Data,
    //         flag: Flag::end_stream(),
    //         id: StreamIdentifier(1),
    //     };
    //     let payload = Payload::Data { data: buf };
    //     let frame = Frame { header, payload };
    //     Ok(Some(frame))
    // }

    // pub fn build_header_frame<T: serialize::Serialize>(
    //     res: &mut Response<T>,
    // ) -> WebResult<Frame<BinaryMut>> {
    //     let mut buf = BinaryMut::new();
    //     let mut enocder = res.get_encoder();
    //     let status = res.status().build_header();
    //     enocder
    //         .encode_header_into((&status.0, &status.1), &mut buf)
    //         .map_err(WebError::from)?;
    //     enocder.encode_into(res.headers().iter(), &mut buf)?;
    //     let header = FrameHeader {
    //         length: buf.remaining() as u32,
    //         kind: Kind::Headers,
    //         flag: Flag::end_headers() | Flag::end_stream(),
    //         id: StreamIdentifier(1),
    //     };
    //     let payload = Payload::Headers {
    //         priority: None,
    //         block: buf,
    //     };
    //     let frame = Frame { header, payload };
    //     Ok(frame)
    // }

    // pub fn build_response_frame<T: serialize::Serialize>(
    //     res: &mut Response<T>,
    // ) -> WebResult<Vec<Frame<BinaryMut>>> {
    //     let mut result = vec![];
    //     result.push(Self::build_header_frame(res)?);
    //     if let Some(frame) = Self::build_body_frame(res)? {
    //         result.push(frame);
    //     }
    //     Ok(result)
    // }

    pub fn serialize<T: serialize::Serialize>(
        res: &mut Response<T>,
        buffer: &mut BinaryMut,
    ) -> WebResult<()> {
        // let vecs = Self::build_response_frame(res)?;
        // for vec in vecs {
        //     vec.serialize(buffer);
        // }
        Ok(())
    }
}
