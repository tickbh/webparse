use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, cell::RefCell, rc::Rc, mem::MaybeUninit, ops::{Deref, DerefMut}, cmp, hash, borrow::{Borrow, BorrowMut}, fmt, vec::IntoIter, ptr, slice, io::{Read, Result, Write}};

use crate::{Buf, Binary, MarkBuf};

use super::BufMut;

/// 二进制的封装, 可写可读
pub struct BinaryMut {
    ptr: *mut Vec<u8>,
    // 共享引用计数
    counter: Rc<RefCell<AtomicUsize>>,
    // 游标值, 可以得出当前指向的位置
    cursor: usize,
    // 标记值, 从上一次标记到现在的游标值, 可以得出偏移的对象
    mark: usize,
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
            mark: 0,
            counter: Rc::new(RefCell::new(AtomicUsize::new(1))),
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
        println!("value = {}",  (*self.counter).borrow().load(std::sync::atomic::Ordering::SeqCst));
        (*self.counter).borrow().load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn as_slice_all(&self) -> &[u8] {
        unsafe {
            &(*self.ptr)[..]
        }
    }

    #[inline]
    pub fn into_slice_all(&self) -> Vec<u8> {
        if (*self.counter).borrow().load(Ordering::SeqCst) == 1 {
            (*self.counter).borrow().fetch_add(1, Ordering::Relaxed);
            let vec = unsafe { Box::from_raw(self.ptr) };
            *vec
        } else {
            unsafe {
                (*self.ptr).clone()
            }
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            &(*self.ptr)[self.cursor..]
        }
    }

    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { &mut (*self.ptr)[self.cursor..] }
    }


    #[inline]
    unsafe fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.remaining() >= by, "internal: inc_start out of bounds");
        self.cursor += by;
    }

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
        unsafe {
            (*self.ptr).len() - self.cursor
        }
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.cursor
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
        unsafe {
            (*self.ptr).capacity()
        }
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


    fn put<T: crate::Buf>(&mut self, mut src: T)
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

    pub fn put_slice(&mut self, src: &[u8]) {
        self.extend_from_slice(src);
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

}

impl From<Vec<u8>> for BinaryMut {
    fn from(value: Vec<u8>) -> Self {
        BinaryMut::from_vec(value)
    }
}

impl Clone for BinaryMut {
    fn clone(&self) -> Self {
        (*self.counter).borrow().fetch_add(1, std::sync::atomic::Ordering::Acquire);
        Self {
            ptr: self.ptr.clone(), 
            cursor: self.cursor.clone(),
            mark: self.mark.clone(),
            counter: self.counter.clone()
        }
    }
}

impl Drop for BinaryMut {
    fn drop(&mut self) {
        println!("drop === {:?} -----", self.counter);

        if (*self.counter).borrow_mut().fetch_sub(1, Ordering::Release) == 1 {
            let _vec = unsafe { Box::from_raw(self.ptr) };
        }

        println!("drop end!!!!");
    }
}


impl Buf for BinaryMut {
    fn remaining(&self) -> usize {
        unsafe {
            (*self.ptr).len() - self.cursor
        }
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, n: usize) {
        unsafe {
            self.inc_start(n);
        }
    }


}



impl MarkBuf for BinaryMut {
    fn mark_commit(&mut self) -> usize {
        self.mark = self.cursor;
        self.mark
    }

    fn mark_slice_skip(&mut self, skip: usize) -> &[u8] {
        debug_assert!(self.cursor - skip >= self.mark);
        let cursor = self.cursor;
        let start = self.mark;
        self.mark_commit();
        let head = &self.as_slice_all()[start .. (cursor - skip)];
        head
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

impl From<BinaryMut> for Binary {
    fn from(src: BinaryMut) -> Binary {
        src.freeze()
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

// impl Borrow<[u8]> for BinaryMut {
//     fn borrow(&self) -> &[u8] {
//         self.as_ref()
//     }
// }

// impl BorrowMut<[u8]> for BinaryMut {
//     fn borrow_mut(&mut self) -> &mut [u8] {
//         self.as_mut()
//     }
// }
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

// impl IntoIterator for BinaryMut {
//     type Item = u8;
//     type IntoIter = IntoIter<BinaryMut>;

//     fn into_iter(self) -> Self::IntoIter {
//         IntoIter::new(self)
//     }
// }

// impl<'a> IntoIterator for &'a BinaryMut {
//     type Item = &'a u8;
//     type IntoIter = core::slice::Iter<'a, u8>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.as_ref().iter()
//     }
// }

// impl Extend<u8> for BinaryMut {
//     fn extend<T>(&mut self, iter: T)
//     where
//         T: IntoIterator<Item = u8>,
//     {
//         let iter = iter.into_iter();

//         let (lower, _) = iter.size_hint();
//         self.reserve(lower);

//         // TODO: optimize
//         // 1. If self.kind() == KIND_VEC, use Vec::extend
//         // 2. Make `reserve` inline-able
//         for b in iter {
//             self.reserve(1);
//             self.put_u8(b);
//         }
//     }
// }

// impl<'a> Extend<&'a u8> for BinaryMut {
//     fn extend<T>(&mut self, iter: T)
//     where
//         T: IntoIterator<Item = &'a u8>,
//     {
//         self.extend(iter.into_iter().copied())
//     }
// }

// impl Extend<Bytes> for BinaryMut {
//     fn extend<T>(&mut self, iter: T)
//     where
//         T: IntoIterator<Item = Bytes>,
//     {
//         for bytes in iter {
//             self.extend_from_slice(&bytes)
//         }
//     }
// }


impl Read for BinaryMut {
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


#[cfg(test)]
mod tests {

}