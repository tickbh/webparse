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
// Created Date: 2023/08/21 11:03:20

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Flag: u8 {
        const END_STREAM = 0x1;
        const ACK = 0x1;
        const END_HEADERS = 0x4;
        const PADDED = 0x8;
        const PRIORITY = 0x20;
    }
}

impl Flag {
    pub fn zero() -> Flag {
        Flag::default()
    }
    pub fn new(data: u8) -> Result<Flag, ()> {
        match Flag::from_bits(data) {
            Some(v) => Ok(v),
            None => Err(()),
        }
    }

    pub fn load(mut flag: Flag) -> Flag {
        flag.set(Flag::ACK, true);
        flag
    }

    pub fn ack() -> Flag {
        Flag::ACK
    }
    pub fn is_ack(&self) -> bool {
        self.contains(Flag::ACK)
    }
    pub fn end_stream() -> Flag {
        Flag::END_STREAM
    }
    pub fn is_end_stream(&self) -> bool {
        self.contains(Flag::END_STREAM)
    }
    pub fn end_headers() -> Flag {
        Flag::END_HEADERS
    }
    pub fn is_end_headers(&self) -> bool {
        self.contains(Flag::END_HEADERS)
    }
    pub fn set_end_headers(&mut self) {
        self.set(Flag::END_HEADERS, true)
    }
    pub fn unset_end_headers(&mut self) {
        self.set(Flag::END_HEADERS, false)
    }
    pub fn padded() -> Flag {
        Flag::PADDED
    }
    pub fn is_padded(&self) -> bool {
        self.contains(Flag::PADDED)
    }
    pub fn set_padded(&mut self) {
        self.set(Flag::PADDED, true)
    }
    pub fn unset_padded(&mut self) {
        self.set(Flag::PADDED, false)
    }
    pub fn priority() -> Flag {
        Flag::PRIORITY
    }
    pub fn is_priority(&self) -> bool {
        self.contains(Flag::PRIORITY)
    }
    pub fn set_end_stream(&mut self) {
        self.set(Flag::END_STREAM, true)
    }
    pub fn unset_end_stream(&mut self) {
        self.set(Flag::END_STREAM, false)
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self(Default::default())
    }
}
