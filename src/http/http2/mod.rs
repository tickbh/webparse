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
// Created Date: 2023/08/21 10:44:37

pub const HTTP2_MAGIC: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
pub const MAIGC_LEN: usize = HTTP2_MAGIC.len();

pub const MAX_WINDOW_SIZE: WindowSize = (1 << 31) - 1; // i32::MAX as u32
pub const DEFAULT_REMOTE_RESET_STREAM_MAX: usize = 20;
pub const DEFAULT_RESET_STREAM_MAX: usize = 10;
pub const DEFAULT_RESET_STREAM_SECS: u64 = 30;
pub const DEFAULT_MAX_SEND_BUFFER_SIZE: usize = 1024 * 400;

/// 默认的header最大长度值
pub const DEFAULT_SETTINGS_HEADER_TABLE_SIZE: usize = 4_096;

/// 默认的发送窗口大小值
pub const DEFAULT_INITIAL_WINDOW_SIZE: u32 = 65_535;

/// 默认的单帧大小
pub const DEFAULT_MAX_FRAME_SIZE: FrameSize = 16_384;

/// 最大的接收端口大小，累加值达到这个数则关闭连接
pub const MAX_INITIAL_WINDOW_SIZE: usize = (1 << 31) - 1;

/// 最大的单帧大小
pub const MAX_MAX_FRAME_SIZE: FrameSize = (1 << 24) - 1;

mod error;
pub mod frame;
mod hpack;

pub use error::Http2Error;
pub use hpack::*;

pub type FrameSize = u32;
pub type WindowSize = u32;

