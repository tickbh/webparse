use std::io::{Read, Write, Result};
use std::ptr;
use std::fmt;
use std::cmp;

use log::{warn, info, trace};

pub struct Buffer {
    val: Vec<u8>,
    start: usize,
    end: usize,
    cursor: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        let mut vec = Vec::with_capacity(512);
        vec.resize(512, 0);
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
            end: 0,
            cursor: 0,
        }
    }
    
    pub fn new_vec(vec: Vec<u8>) -> Buffer {
        Buffer {
            val: vec,
            start: 0,
            end: 0,
            cursor: 0,
        }
    }
    
    pub fn get_data(&self) -> &Vec<u8> {
        &self.val
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

    pub fn set_rpos(&mut self, rpos: usize) {
        self.start = rpos;
    }

    pub fn get_rpos(&self) -> usize {
        self.start
    }

    pub fn set_wpos(&mut self, wpos: usize) {
        self.end = wpos;
    }

    pub fn get_wpos(&self) -> usize {
        self.end
    }
    
    pub fn get_read_array(&self, max_bytes: usize) -> &[u8] {
        &self.val[self.end .. (self.end+max_bytes-1)]
    }
    
    pub fn drain(&mut self, pos: usize) {
        self.start = self.start - cmp::min(self.start, pos);
        self.end = self.end - cmp::min(self.end, pos);
        let pos = cmp::min(self.val.len(), pos);
        self.val.drain(..pos);
        self.fix_buffer();
    }

    pub fn drain_collect(&mut self, pos: usize) -> Vec<u8> {
        self.start = self.start - cmp::min(self.start, pos);
        self.end = self.end - cmp::min(self.end, pos);
        let pos = cmp::min(self.val.len(), pos);
        let ret = self.val.drain(..pos).collect();
        self.fix_buffer();
        ret
    }
    
    pub fn drain_all_collect(&mut self) -> Vec<u8> {
        let (rpos, wpos) = (self.start, self.end);
        self.clear();
        self.val.drain(rpos..wpos).collect()
    }

    pub fn fix_buffer(&mut self) -> bool {
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
    
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if self.start < self.end {
            Some(self.val[self.start])
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
        self.commit();
        let head = &self.val[self.start .. cursor - skip];
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
    pub fn slice(&mut self) -> &[u8] {
        let cursor = self.cursor;
        self.commit();
        let slice = &self.val[self.start .. cursor];
        slice
    }


}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "bytes ({:?})", self.val)
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
