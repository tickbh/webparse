pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
pub const MAIGC_LEN: usize = HTTP2_MAGIC.len();

use std::{borrow::Cow, fmt::Debug};
mod error;
pub mod frame;
mod hpack;

pub use error::Http2Error;

use crate::{
    http::http2::frame::Settings, serialize, Binary, BinaryMut, Buf, BufMut, MarkBuf, Method,
    Request, Response, Serialize, WebError, WebResult,
};
pub use hpack::*;

use self::frame::{Flag, Frame, FrameHeader, Kind, StreamIdentifier};


pub type FrameSize = u32;
pub type WindowSize = u32;

// Constants
pub const MAX_WINDOW_SIZE: WindowSize = (1 << 31) - 1; // i32::MAX as u32
pub const DEFAULT_REMOTE_RESET_STREAM_MAX: usize = 20;
pub const DEFAULT_RESET_STREAM_MAX: usize = 10;
pub const DEFAULT_RESET_STREAM_SECS: u64 = 30;
pub const DEFAULT_MAX_SEND_BUFFER_SIZE: usize = 1024 * 400;

/// The default value of SETTINGS_HEADER_TABLE_SIZE
pub const DEFAULT_SETTINGS_HEADER_TABLE_SIZE: usize = 4_096;

/// The default value of SETTINGS_INITIAL_WINDOW_SIZE
pub const DEFAULT_INITIAL_WINDOW_SIZE: u32 = 65_535;

/// The default value of MAX_FRAME_SIZE
pub const DEFAULT_MAX_FRAME_SIZE: FrameSize = 16_384;

/// INITIAL_WINDOW_SIZE upper bound
pub const MAX_INITIAL_WINDOW_SIZE: usize = (1 << 31) - 1;

/// MAX_FRAME_SIZE upper bound
pub const MAX_MAX_FRAME_SIZE: FrameSize = (1 << 24) - 1;
