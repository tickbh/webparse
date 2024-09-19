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
// Created Date: 2023/08/18 02:58:54

use algorithm::buf::{Bt, BtMut, Binary, BinaryMut};

use crate::WebResult;

pub trait Serialize {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize>;
}


impl Serialize for &'static str {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for String {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(self.as_bytes()))
    }
}

impl Serialize for () {
    fn serialize<B: Bt+BtMut>(&mut self, _buffer: &mut B) -> WebResult<usize> {
        Ok(0)
    }
}

impl Serialize for Vec<u8> {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(&self))
    }
}

impl Serialize for &[u8] {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        Ok(buffer.put_slice(&self))
    }
}

impl Serialize for Binary {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let len = self.remaining();
        buffer.put_slice(self.chunk());
        Ok(len)
    }
}

impl Serialize for BinaryMut {
    fn serialize<B: Bt+BtMut>(&mut self, buffer: &mut B) -> WebResult<usize> {
        let len = self.remaining();
        buffer.put_slice(self.chunk());
        Ok(len)
    }
}
