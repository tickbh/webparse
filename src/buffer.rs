use std::io::{Read, Write, Result};
use std::ptr;
use std::fmt;
use std::cmp;

use log::{warn, info, trace};

pub struct Buffer {
    val: Vec<u8>,
    rpos: usize,
    wpos: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        let mut vec = Vec::with_capacity(512);
        vec.resize(512, 0);
        Buffer {
            val: vec,
            rpos: 0,
            wpos: 0,
        }
    }

    pub fn new_buf(buf: &[u8]) -> Buffer {
        let vec = Vec::from(buf);
        Buffer {
            val: vec,
            rpos: 0,
            wpos: 0,
        }
    }
    
    pub fn new_vec(vec: Vec<u8>) -> Buffer {
        Buffer {
            val: vec,
            rpos: 0,
            wpos: 0,
        }
    }
    
    pub fn get_data(&self) -> &Vec<u8> {
        &self.val
    }

    pub fn get_write_data(&self) -> &[u8] {
        &self.val[self.rpos .. self.wpos]
    }
    
    pub fn cache_len(&self) -> usize {
        self.val.len()
    }

    pub fn data_len(&self) -> usize {
        if self.wpos > self.rpos {
            (self.wpos - self.rpos) as usize
        } else {
            0
        }
    }

    pub fn set_rpos(&mut self, rpos: usize) {
        self.rpos = rpos;
    }

    pub fn get_rpos(&self) -> usize {
        self.rpos
    }

    pub fn set_wpos(&mut self, wpos: usize) {
        self.wpos = wpos;
    }

    pub fn get_wpos(&self) -> usize {
        self.wpos
    }
    
    pub fn get_read_array(&self, max_bytes: usize) -> &[u8] {
        &self.val[self.wpos .. (self.wpos+max_bytes-1)]
    }
    
    pub fn drain(&mut self, pos: usize) {
        self.rpos = self.rpos - cmp::min(self.rpos, pos);
        self.wpos = self.wpos - cmp::min(self.wpos, pos);
        let pos = cmp::min(self.val.len(), pos);
        self.val.drain(..pos);
        self.fix_buffer();
    }

    pub fn drain_collect(&mut self, pos: usize) -> Vec<u8> {
        self.rpos = self.rpos - cmp::min(self.rpos, pos);
        self.wpos = self.wpos - cmp::min(self.wpos, pos);
        let pos = cmp::min(self.val.len(), pos);
        let ret = self.val.drain(..pos).collect();
        self.fix_buffer();
        ret
    }
    
    pub fn drain_all_collect(&mut self) -> Vec<u8> {
        let (rpos, wpos) = (self.rpos, self.wpos);
        self.clear();
        self.val.drain(rpos..wpos).collect()
    }

    pub fn fix_buffer(&mut self) -> bool {
        if self.rpos >= self.wpos {
            self.rpos = 0;
            self.wpos = 0;

            if self.val.len() > 512000 {
                self.val.resize(512000, 0);
                warn!("buffer len big than 512k, resize to 512k");
            } else {
                trace!("read all size, reset to zero");
            }
        } else if self.rpos > self.val.len() / 2 {
            unsafe {
                ptr::copy(&self.val[self.rpos], &mut self.val[0], self.wpos - self.rpos);
                info!("fix buffer {} has half space so move position", self.rpos);
                (self.rpos, self.wpos) = (0, self.wpos - self.rpos);
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.rpos = 0;
        self.wpos = 0;
    }
    
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        if self.rpos < self.wpos {
            Some(self.val[self.rpos])
        } else {
            None
        }
    }

    #[inline]
    pub fn peek_ahead(&self, n: usize) -> Option<u8> {
        if self.rpos > n {
            Some(self.val[self.rpos - n])
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

}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "bytes ({:?})", self.val)
    }
}

impl Read for Buffer {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.wpos - self.rpos;
        if left == 0 || buf.len() == 0 {
            return Ok(0);
        }
        let read = if left > buf.len() {
            buf.len()
        } else {
            left
        };
        unsafe {
            ptr::copy(&self.val[self.rpos], &mut buf[0], read);
        }
        self.rpos += read;
        Ok(read)
    }

    
}

impl Write for Buffer {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.val.len() < self.wpos + buf.len() {
            self.val.resize((self.wpos + buf.len()) * 2, 0);
            if self.val.len() > 512000 {
                warn!("resize buffer length to {:?}k", self.val.len() / 1024);
            }
        }
        if buf.len() == 0 {
            return Ok(buf.len());
        }
        unsafe {
            ptr::copy(&buf[0], &mut self.val[self.wpos], buf.len());
        }
        self.wpos += buf.len();
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
        let left = self.wpos - self.rpos;
        if self.wpos > self.rpos {
            let read = self.val[self.rpos];
            self.rpos += 1;
            return Some(read);
        } else {
            None
        }
    }
}
