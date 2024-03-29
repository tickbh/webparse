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
// Created Date: 2023/08/28 09:38:10

use std::{
    cell::RefCell,
    cmp,
    fmt::{self, Debug},
    hash,
    io::{self, Error, Read, Result, Write},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{Binary, Buf, WebError};

use super::BufMut;

/// 100k，当数据大于100k时，可以尝试重排当前的结构
static RESORT_MEMORY_SIZE: usize = 102400;
/// 二进制的封装, 可写可读
pub struct BinaryMut {
    ptr: *mut Vec<u8>,
    // 共享引用计数
    counter: Rc<RefCell<AtomicUsize>>,
    // 游标值, 可以得出当前指向的位置
    cursor: usize,
    // 手动设置长度, 分片时使用
    manual_len: usize,
    // 标记值, 从上一次标记到现在的游标值, 可以得出偏移的对象
    mark: usize,
    // 尝试重排的大小
    resort: usize,
}

impl BinaryMut {
    #[inline]
    pub fn with_capacity(n: usize) -> BinaryMut {
        BinaryMut::from_vec(Vec::with_capacity(n))
    }

    /// 新建对象
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::BinaryMut;
    ///
    /// let mut bytes = BinaryMut::new();
    /// assert_eq!(0, bytes.len());
    /// bytes.reserve(2);
    /// bytes.put_slice(b"xy");
    /// assert_eq!(&b"xy"[..], &bytes[..]);
    /// ```
    #[inline]
    pub fn new() -> BinaryMut {
        BinaryMut::with_capacity(0)
    }

    #[inline]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BinaryMut {
        let ptr = Box::into_raw(Box::new(vec));
        BinaryMut {
            ptr,
            cursor: 0,
            manual_len: usize::MAX,
            mark: 0,
            counter: Rc::new(RefCell::new(AtomicUsize::new(1))),
            resort: RESORT_MEMORY_SIZE,
        }
    }

    /// 获取引用的数量
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::BinaryMut;
    ///
    /// let b = BinaryMut::new();
    /// {
    /// let b1 = b.clone();
    /// assert!(b1.get_refs() == 2);
    /// drop(b1);
    /// }
    /// assert!(b.get_refs() == 1);
    /// ```
    pub fn get_refs(&self) -> usize {
        (*self.counter)
            .borrow()
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn into_slice_all(&self) -> Vec<u8> {
        if (*self.counter).borrow().load(Ordering::SeqCst) == 1 {
            (*self.counter).borrow().fetch_add(1, Ordering::Relaxed);
            let vec = unsafe { Box::from_raw(self.ptr) };
            *vec
        } else {
            unsafe { (*self.ptr).clone() }
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let end = std::cmp::min(self.manual_len, (*self.ptr).len());
            &(*self.ptr)[self.cursor..end]
        }
    }

    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            let end = std::cmp::min(self.manual_len, (*self.ptr).len());
            &mut (*self.ptr)[self.cursor..end]
        }
    }

    #[inline]
    unsafe fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.remaining() >= by, "internal: inc_start out of bounds");
        self.cursor += by;
    }

    // #[inline]
    // unsafe fn sub_start(&mut self, by: usize) {
    //     // should already be asserted, but debug assert for tests
    //     debug_assert!(self.cursor >= by, "internal: sub_start out of bounds");
    //     self.cursor -= by;
    // }

    /// 判断对象的长度
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::BinaryMut;
    ///
    /// let b = BinaryMut::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (*self.ptr).len() - self.cursor }
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cursor = 0;
        unsafe {
            (*self.ptr).set_len(0);
        }
    }
    /// 判断对象是否为空
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::BinaryMut;
    ///
    /// let b = BinaryMut::with_capacity(64);
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 返回对象大小的容量
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::BinaryMut;
    ///
    /// let b = BinaryMut::with_capacity(64);
    /// assert_eq!(b.capacity(), 64);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        unsafe { (*self.ptr).capacity() }
    }

    pub fn reserve(&mut self, additional: usize) {
        unsafe {
            let len = (*self.ptr).len();
            let rem = (*self.ptr).capacity() - len;
            if rem >= additional {
                return;
            }
            (*self.ptr).reserve(additional)
        }
    }

    pub fn put<T: crate::Buf>(&mut self, mut src: T)
    where
        Self: Sized,
    {
        while src.has_remaining() {
            let s = src.chunk();
            let l = s.len();
            self.extend_from_slice(s);
            src.advance(l);
        }
    }

    pub fn put_slice(&mut self, src: &[u8]) -> usize {
        self.extend_from_slice(src);
        src.len()
    }

    /// 将当前的数据转成不可写的对象Binary
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BinaryMut;
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// let bin = buf.freeze();
    ///
    /// assert_eq!(b"aaabbb", &bin[..]);
    /// ```
    #[inline]
    pub fn freeze(self) -> Binary {
        Binary::from(self.into_slice_all())
    }

    pub fn copy_to_binary(&mut self) -> Binary {
        let binary = Binary::from(self.chunk().to_vec());
        self.advance_all();
        binary
    }

    /// 扩展bytes到`BinaryMut`, 将会自动扩展容量空间
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BinaryMut;
    ///
    /// let mut buf = BinaryMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// buf.extend_from_slice(b"cccddd");
    ///
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        let cnt = extend.len();
        self.reserve(cnt);

        unsafe {
            let dst = self.chunk_mut();
            // Reserved above
            debug_assert!(dst.len() >= cnt);

            ptr::copy_nonoverlapping(extend.as_ptr(), dst.as_mut_ptr().cast(), cnt);
        }

        unsafe {
            self.advance_mut(cnt);
        }
    }

    pub fn get_resort(&self) -> usize {
        self.resort
    }
    
    pub fn set_resort(&mut self, resort: usize) {
        self.resort = resort;
    }

    #[inline]
    pub unsafe fn try_resort_memory(&mut self) {
        if (*self.ptr).len() < self.resort || self.cursor < self.resort / 2 {
            return;
        }
        let left = self.remaining();
        // 只有当前只有一个引用的时候尝试做数据迁移，否则会影响另外的数据
        if (*self.counter).borrow().load(Ordering::SeqCst) == 1 {
            if left == 0 {
                (*self.ptr).set_len(0);
            } else {
                std::ptr::copy((*self.ptr).as_ptr().add(self.cursor), (*self.ptr).as_mut_ptr(), left);
                (*self.ptr).set_len(left);
            }

            self.cursor = 0;
            if self.manual_len != usize::MAX {
                self.manual_len = left;
            }
        }
    }
}

impl From<Vec<u8>> for BinaryMut {
    fn from(value: Vec<u8>) -> Self {
        BinaryMut::from_vec(value)
    }
}

impl Clone for BinaryMut {
    fn clone(&self) -> Self {
        (*self.counter)
            .borrow()
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        Self {
            ptr: self.ptr.clone(),
            cursor: self.cursor.clone(),
            manual_len: self.manual_len,
            mark: self.mark.clone(),
            counter: self.counter.clone(),
            resort: self.resort,
        }
    }
}

impl Drop for BinaryMut {
    fn drop(&mut self) {
        if (*self.counter).borrow_mut().fetch_sub(1, Ordering::Release) == 1 {
            let _vec = unsafe { Box::from_raw(self.ptr) };
        }
    }
}

impl Buf for BinaryMut {
    fn remaining(&self) -> usize {
        unsafe {
            std::cmp::min(self.manual_len, (*self.ptr).len()) - self.cursor
        }
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance_chunk(&mut self, n: usize) -> &[u8] {
        let ret = &unsafe {
            let end = std::cmp::min(self.manual_len, (*self.ptr).len());
            &(*self.ptr)[self.cursor..end]
        }[..n];
        self.advance(n);
        ret
    }
    
    fn advance(&mut self, n: usize) {
        unsafe {
            self.inc_start(n);
            self.try_resort_memory();
        }
    }

    fn into_binary(self) -> Binary {
        Binary::from(self.chunk().to_vec())
    }

}

unsafe impl BufMut for BinaryMut {
    fn remaining_mut(&self) -> usize {
        usize::MAX - self.len()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        let len = (*self.ptr).len();
        (*self.ptr).set_len(len + cnt);
    }

    fn chunk_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            if (*self.ptr).len() == (*self.ptr).capacity() {
                self.reserve(128);
            }
            (*self.ptr).spare_capacity_mut()
        }
    }
}

impl AsRef<[u8]> for BinaryMut {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for BinaryMut {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsMut<[u8]> for BinaryMut {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl DerefMut for BinaryMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> From<&'a [u8]> for BinaryMut {
    fn from(src: &'a [u8]) -> BinaryMut {
        BinaryMut::from_vec(src.to_vec())
    }
}

impl<'a> From<&'a str> for BinaryMut {
    fn from(src: &'a str) -> BinaryMut {
        BinaryMut::from(src.as_bytes())
    }
}

impl From<String> for BinaryMut {
    fn from(src: String) -> BinaryMut {
        BinaryMut::from_vec(src.into_bytes())
    }
}

impl From<BinaryMut> for Binary {
    fn from(src: BinaryMut) -> Binary {
        src.freeze()
    }
}

impl From<Binary> for BinaryMut {
    fn from(src: Binary) -> BinaryMut {
        BinaryMut::from(src.into_slice())
    }
}

impl PartialEq for BinaryMut {
    fn eq(&self, other: &BinaryMut) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for BinaryMut {
    fn partial_cmp(&self, other: &BinaryMut) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl Ord for BinaryMut {
    fn cmp(&self, other: &BinaryMut) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Eq for BinaryMut {}

impl Default for BinaryMut {
    #[inline]
    fn default() -> BinaryMut {
        BinaryMut::new()
    }
}

impl hash::Hash for BinaryMut {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

impl Iterator for BinaryMut {
    type Item = u8;
    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.get_next()
    }
}

impl fmt::Write for BinaryMut {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.remaining_mut() >= s.len() {
            self.put_slice(s.as_bytes());
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl TryInto<String> for BinaryMut {
    type Error = WebError;

    fn try_into(self) -> std::result::Result<String, Self::Error> {
        Ok(String::from_utf8_lossy(&self.chunk()).to_string())
    }
}

impl Read for BinaryMut {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.remaining();
        if left == 0 || buf.len() == 0 {
            return Err(Error::new(io::ErrorKind::WouldBlock, ""));
        }
        let read = std::cmp::min(left, buf.len());
        unsafe {
            std::ptr::copy(&self.chunk()[0], &mut buf[0], read);
        }
        self.advance(read);
        Ok(read)
    }
}

impl Write for BinaryMut {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.put_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Debug for BinaryMut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryMut")
            .field("ptr", &self.ptr)
            .field("counter", &self.counter)
            .field("cursor", &self.cursor)
            .field("manual_len", &self.manual_len)
            .field("mark", &self.mark)
            .finish()
    }
}

unsafe impl Sync for BinaryMut {}
unsafe impl Send for BinaryMut {}

#[cfg(test)]
mod tests {}
