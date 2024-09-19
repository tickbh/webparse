// Copyright 2022 - 2024 Wenmeng See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Author: tickbh
// -----
// Created Date: 2023/12/29 02:00:25

use algorithm::buf::BtMut;

/// Struct to pipe data into another writer,
/// while masking the data being written
pub struct Masker<'w> {
	key: [u8; 4],
	pos: usize,
	end: &'w mut dyn BtMut,
}

impl<'w> Masker<'w> {
	/// Create a new Masker with the key and the endpoint
	/// to be writer to.
	pub fn new(key: [u8; 4], endpoint: &'w mut dyn BtMut) -> Self {
		Masker {
			key,
			pos: 0,
			end: endpoint,
		}
	}

    // fn write(&mut self, data: &[u8]) -> usize {
	// 	let mut buf = Vec::with_capacity(data.len());
	// 	for &byte in data.iter() {
	// 		buf.push(byte ^ self.key[self.pos]);
	// 		self.pos = (self.pos + 1) % self.key.len();
	// 	}
	// 	self.end.put_slice(&buf)
	// }
}

unsafe impl<'w> BtMut for Masker<'w> {
    fn remaining_mut(&self) -> usize {
        self.end.remaining_mut()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.end.advance_mut(cnt)
    }

    fn chunk_mut(&mut self) -> &mut [std::mem::MaybeUninit<u8>] {
        self.end.chunk_mut()
    }

    fn put_slice(&mut self, src: &[u8]) -> usize {
        let mut buf = Vec::with_capacity(src.len());
		for &byte in src.iter() {
			buf.push(byte ^ self.key[self.pos]);
			self.pos = (self.pos + 1) % self.key.len();
		}
		self.inner_put_slice(&buf)
    }
}

/// Masks data to send to a server and writes
pub fn mask_data(mask: [u8; 4], data: &[u8]) -> Vec<u8> {
	let mut out = Vec::with_capacity(data.len());
	let zip_iter = data.iter().zip(mask.iter().cycle());
	for (&buf_item, &key_item) in zip_iter {
		out.push(buf_item ^ key_item);
	}
	out
}

#[cfg(test)]
mod tests {
	
	use test;
	use super::*;

	#[test]
	fn test_mask_data() {
		let key = [1u8, 2u8, 3u8, 4u8];
		let original = vec![10u8, 11u8, 12u8, 13u8, 14u8, 15u8, 16u8, 17u8];
		let expected = vec![11u8, 9u8, 15u8, 9u8, 15u8, 13u8, 19u8, 21u8];
		let obtained = mask_data(key, &original[..]);
		let reversed = mask_data(key, &obtained[..]);

		assert_eq!(original, reversed);
		assert_eq!(obtained, expected);
	}

}
