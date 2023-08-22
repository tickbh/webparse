use std::io::{Read, Write, Result};
use std::ptr;
use std::fmt;

use log::{warn, info, trace};

pub struct Buffer {
    val: Vec<u8>,
    start: usize,
    end: usize,
    cursor: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        let mut vec = Vec::with_capacity(1024);
        vec.resize(1024, 0);
        Buffer {
            val: vec,
            start: 0,
            end: 0,
            cursor: 0,
        }
    }

    pub fn new_buf(buf: &[u8]) -> Buffer {
        let vec = Vec::from(buf);
        Buffer {
            val: vec,
            start: 0,
            end: buf.len(),
            cursor: 0,
        }
    }
    
    pub fn new_vec(vec: Vec<u8>) -> Buffer {
        let len = vec.len();
        Buffer {
            val: vec,
            start: 0,
            end: len,
            cursor: 0,
        }
    }
    
    pub fn get_write_data(&self) -> &[u8] {
        &self.val[self.start .. self.end]
    }
    
    pub fn cache_len(&self) -> usize {
        self.val.len()
    }

    pub fn data_len(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }

    pub fn set_start(&mut self, start: usize) {
        self.start = start;
    }


    pub fn get_start(&self) -> usize {
        self.start
    }

    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    pub fn get_end(&self) -> usize {
        self.end
    }
    
    pub fn add_write_len(&mut self, len: usize) {
        self.end += len;   
    }
    
    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub fn get_cursor(&self) -> usize {
        self.cursor
    }
    
    pub fn is_end(&self) -> bool {
        self.cursor == self.end || self.start == self.end
    }

    pub fn now(&self) -> u8 {
        self.val[self.cursor]
    }

    pub fn get_read_array(&self, max_bytes: usize) -> &[u8] {
        &self.val[self.cursor .. std::cmp::min(self.end, self.cursor+max_bytes-1)]
    }
    
    pub fn get_left_array(&self) -> &[u8] {
        &self.val[self.cursor .. self.end]
    }
    
    pub fn get_write_array(&mut self, write_bytes: usize) -> &mut [u8] {
        if self.end + write_bytes > self.val.len() {
            self.val.resize(self.end + write_bytes + 128, 0);
        }
        &mut self.val[self.end .. (self.end+write_bytes-1)]
    }

    pub fn write_data(self) -> Vec<u8> {
        self.val[self.start .. self.end].to_vec()
    }

    pub fn refresh(&mut self) -> bool {
        if self.start >= self.end {
            self.start = 0;
            self.end = 0;

            if self.val.len() > 512000 {
                self.val.resize(512000, 0);
                warn!("buffer len big than 512k, resize to 512k");
            } else {
                trace!("read all size, reset to zero");
            }
        } else if self.start > self.val.len() / 2 {
            unsafe {
                ptr::copy(&self.val[self.start], &mut self.val[0], self.end - self.start);
                info!("fix buffer {} has half space so move position", self.start);
                (self.start, self.end) = (0, self.end - self.start);
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub fn commit(&mut self) {
        self.start = self.cursor
    }

    pub fn uncommit(&mut self) {
        self.cursor = self.start
    }
    
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if self.cursor < self.end {
            Some(self.val[self.cursor])
        } else {
            None
        }
    }

    #[inline]
    pub fn peek_ahead(&self, n: usize) -> Option<u8> {
        if self.start > n {
            Some(self.val[self.start - n])
        } else {
            None
        }
    }

    #[inline]
    pub fn peek_n<'a, U: TryFrom<&'a [u8]>>(&'a self, n: usize) -> Option<U> {
        if self.data_len() < n {
            None
        } else {
            self.get_read_array(n).try_into().ok()
        }
    }

        #[inline]
    pub fn slice_skip(&mut self, skip: usize) -> &[u8] {
        debug_assert!(self.cursor - skip >= self.start);
        let cursor = self.cursor;
        let start = self.start;
        self.commit();
        let head = &self.val[start .. (cursor - skip)];
        head
    }

    #[inline]
    pub fn bump(&mut self) {
        self.advance(1)
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.cursor = self.cursor + n;
        debug_assert!(self.cursor <= self.end, "overflow");
    }
    
    #[inline]
    pub fn retreat(&mut self, n: usize) {
        self.cursor = self.cursor - n;
        debug_assert!(self.cursor >= self.start, "overflow");
    }
    

    #[inline]
    pub fn slice(&mut self) -> &[u8] {
        let cursor = self.cursor;
        let start = self.start;
        self.commit();
        let slice = &self.val[start .. cursor];
        slice
    }

    pub fn write_u8(&mut self, b: u8) -> Result<usize> {
        self.write(&[b])
    }

    pub fn bit_iter<'a>(&'a mut self, len: Option<usize>) -> BitIterator {
        match len {
            Some(l) => BitIterator::new(self, std::cmp::min(self.cursor + l, self.end)),
            None => BitIterator::new(self, self.end),
        }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "bytes ({:?})", self.get_write_data())
    }
}

impl Read for Buffer {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.end - self.start;
        if left == 0 || buf.len() == 0 {
            return Ok(0);
        }
        let read = if left > buf.len() {
            buf.len()
        } else {
            left
        };
        unsafe {
            ptr::copy(&self.val[self.start], &mut buf[0], read);
        }
        self.start += read;
        Ok(read)
    }
}

impl Write for Buffer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.val.len() < self.end + buf.len() {
            self.val.resize((self.end + buf.len()) * 2, 0);
            if self.val.len() > 512000 {
                warn!("resize buffer length to {:?}k", self.val.len() / 1024);
            }
        }
        if buf.len() == 0 {
            return Ok(buf.len());
        }
        unsafe {
            ptr::copy(&buf[0], &mut self.val[self.end], buf.len());
        }
        self.end += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Iterator for Buffer {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.end > self.cursor {
            let read = self.val[self.cursor];
            self.cursor += 1;
            return Some(read);
        } else {
            None
        }
    }
}

pub struct BitIterator<'a> {
    buffer_iterator: &'a mut Buffer,
    current_byte: Option<u8>,
    pos: u8,
    end: usize,
}

impl<'a> BitIterator<'a> {
    pub fn new(iterator: &'a mut Buffer, end: usize) -> BitIterator {
        BitIterator {
            buffer_iterator: iterator,
            current_byte: None,
            pos: 7,
            end,
        }
    }
}

impl<'a> Iterator for BitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.current_byte.is_none() && self.buffer_iterator.get_cursor() < self.end {
            self.current_byte = self.buffer_iterator.next();
            self.pos = 7;
        }

        // If we still have `None`, it means the buffer has been exhausted
        if self.current_byte.is_none() {
            return None;
        }

        let b = self.current_byte.unwrap();

        let is_set = (b & (1 << self.pos)) == (1 << self.pos);
        if self.pos == 0 {
            // We have exhausted all bits from the current byte -- try to get
            // a new one on the next pass.
            self.current_byte = None;
        } else {
            // Still more bits left here...
            self.pos -= 1;
        }

        Some(is_set)
    }
}