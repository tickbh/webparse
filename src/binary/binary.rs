use std::{sync::{Arc, atomic::AtomicUsize}, slice, mem, io::Read, io::Result};

use super::Buf;

static EMPTY_ARRAY: &[u8] = &[];


pub struct Binary {
    pub(crate) ptr: *const u8,
    pub(crate) counter: Arc<AtomicUsize>,
    // 游标可以得出指针的初始位置
    pub(crate) cursor: usize,
    pub(crate) len: usize,
    vtable: &'static Vtable,
}

pub struct Vtable {
    pub clone: unsafe fn(bin: &Binary) -> Binary,
    pub to_vec: unsafe fn(bin: &Binary) -> Vec<u8>,
    pub drop: unsafe fn(bin: &mut Binary),
}


const STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    to_vec: static_to_vec,
    drop: static_drop,
};

unsafe fn static_clone(bin: &Binary) -> Binary {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    Binary::from_static(slice)
}

unsafe fn static_to_vec(bin: &Binary) -> Vec<u8> {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    slice.to_vec()
}

unsafe fn static_drop(_bin: &mut Binary) {
    // nothing to drop for &'static [u8]
}

const SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    to_vec: shared_to_vec,
    drop: shared_drop,
};

unsafe fn shared_clone(bin: &Binary) -> Binary {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    Binary::from_static(slice)
}

unsafe fn shared_to_vec(bin: &Binary) -> Vec<u8> {
    let slice = slice::from_raw_parts(bin.ptr, bin.len);
    slice.to_vec()
}

unsafe fn shared_drop(_bin: &mut Binary) {
    // nothing to drop for &'static [u8]
}
impl Binary {

    pub fn new() -> Binary {
        Binary::from_static(EMPTY_ARRAY)
    }
    
    pub fn from_static(val: &'static [u8]) -> Binary {
        Binary { 
            ptr: val.as_ptr(), 
            counter: Arc::new(AtomicUsize::new(1)), 
            cursor: 0, len: val.len(), 
            vtable: &STATIC_VTABLE
        }
    }

    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::Binary;
    ///
    /// let b = Binary::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    /// 
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the `Bytes` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::Binary;
    ///
    /// let b = Binary::new();
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        unsafe {
            (self.vtable.to_vec)(self)
        }
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    unsafe fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.len >= self.cursor + by, "internal: inc_start out of bounds");
        self.len -= by;
        self.ptr = self.ptr.add(by);
        self.cursor += self.cursor;
    }
}

impl Clone for Binary {
    fn clone(&self) -> Self {
        unsafe {
            (self.vtable.clone)(self)
        }
    }
}

impl Drop for Binary {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.drop)(self)
        }
    }
}


impl From<&'static str> for Binary {
    fn from(value: &'static str) -> Self {
        Binary::from_static(value.as_bytes())
    }
}


impl From<&'static [u8]> for Binary {
    fn from(value: &'static [u8]) -> Self {
        Binary::from_static(value)
    }
}

impl From<Box<[u8]>> for Binary {
    fn from(mut value: Box<[u8]>) -> Self {
        let ptr = value.as_mut_ptr();
        let len = value.len();
        mem::forget(value);

        Binary {
            ptr,
            len,
            cursor: 0,
            counter: Arc::new(AtomicUsize::new(1)),
            vtable: &SHARED_VTABLE,
        }
    }
}

impl From<Vec<u8>> for Binary {
    fn from(value: Vec<u8>) -> Self {
        Binary::from(value.into_boxed_slice())
    }
}


impl Buf for Binary {
    fn remaining(&self) -> usize {
        self.len
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, n: usize) {
        unsafe {
            self.inc_start(n);
        }
    }

    fn slice_skip(&mut self, skip: usize) -> &[u8] {
        todo!()
    }
}

impl Read for Binary {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.remaining();
        if left == 0 || buf.len() == 0 {
            return Ok(0);
        }
        let read = std::cmp::min(left, buf.len());
        unsafe {
            std::ptr::copy(&self.chunk()[0], &mut buf[0], read);
        }
        self.advance(read);
        Ok(read)
    }
}

impl Iterator for Binary {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        if self.has_remaining() {
            let read = self.chunk()[0];
            self.advance(1);
            Some(read)
        } else {
            None
        }
    }
}