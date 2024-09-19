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
// Created Date: 2023/09/01 04:09:08

mod data;
mod flag;
mod frame;
mod go_away;
mod headers;
mod kind;
mod ping;
mod priority;
mod reason;
mod reset;
mod settings;
mod window_update;

use std::{cmp::Ordering, fmt::Display};

pub use data::Data;
pub use flag::Flag;
pub use frame::{Frame, PriorityFrame};
pub use headers::{Headers, PushPromise};
pub use kind::Kind;
pub use priority::{Priority, StreamDependency};

pub use self::go_away::GoAway;
pub use self::ping::Ping;
pub use self::reason::Reason;
pub use self::reset::Reset;
pub use self::settings::Settings;
pub use self::window_update::WindowUpdate;

use algorithm::buf::{Bt, BtMut};
use crate::{Serialize, WebResult};

pub use self::frame::FrameHeader;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StreamIdentifier(pub u32);

impl StreamIdentifier {
    pub fn parse<T: Bt>(buf: &mut T) -> StreamIdentifier {
        if buf.remaining() < 4 {
            return StreamIdentifier(0);
        }
        StreamIdentifier(read_u31(buf))
    }

    pub fn zero() -> StreamIdentifier {
        StreamIdentifier(0)
    }

    pub fn client_first() -> StreamIdentifier {
        StreamIdentifier(1)
    }

    pub fn server_first() -> StreamIdentifier {
        StreamIdentifier(2)
    }

    pub fn next_id(&mut self) -> StreamIdentifier {
        let now = self.0;
        self.0 = self.0 + 2;
        StreamIdentifier(now)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn encode<B: Bt + BtMut>(&self, buffer: &mut B) -> WebResult<usize> {
        buffer.put_u32(self.0);
        Ok(4)
    }
}

impl From<u32> for StreamIdentifier {
    fn from(value: u32) -> Self {
        StreamIdentifier(value)
    }
}

impl Ord for StreamIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for StreamIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for StreamIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

#[inline(always)]
pub fn read_u64<T: Bt>(buf: &mut T) -> u64 {
    if buf.remaining() < 8 {
        return 0;
    }
    buf.get_u64()
}

pub const MASK_U31: u32 = (1u32 << 31) - 1;
#[inline(always)]
pub fn read_u31<T: Bt>(buf: &mut T) -> u32 {
    if buf.remaining() < 4 {
        return 0;
    }
    let val = buf.get_u32();
    val & MASK_U31
    // (buf[0] as u32 & 0x7F) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | buf[3] as u32
}

#[inline(always)]
pub fn read_u24<T: Bt>(buf: &mut T) -> u32 {
    if buf.remaining() < 3 {
        return 0;
    }
    (buf.get_u8() as u32) << 16 | (buf.get_u8() as u32) << 8 | buf.get_u8() as u32
}

#[inline(always)]
pub fn encode_u24<B: Bt + BtMut>(buf: &mut B, val: u32) -> usize {
    buf.put_u8((val >> 16) as u8);
    buf.put_u8((val >> 8) as u8);
    buf.put_u8((val >> 0) as u8);
    3
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SizeIncrement(pub u32);

impl SizeIncrement {
    pub fn parse<T: Bt>(buf: &mut T) -> SizeIncrement {
        SizeIncrement(buf.get_u32())
    }

    pub fn encode<B: Bt + BtMut>(&self, buf: &mut B) -> usize {
        buf.put_u32(self.0);
        4
    }
}

impl Serialize for SizeIncrement {
    fn serialize<B: Bt + BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_u32(self.0))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParserSettings {
    padding: bool,
    priority: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub fn parse<T: Bt>(buf: &mut T) -> ErrorCode {
        buf.advance(4);
        ErrorCode(0)
    }
}

impl Serialize for ErrorCode {
    fn serialize<B: Bt + BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        buffer.put_u32(self.0);
        Ok(4)
    }
}
