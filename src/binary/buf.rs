use std::{mem, ops::{Range, RangeBounds}};

macro_rules! buf_get_impl {
    ($this:ident, $typ:tt::$conv:tt) => {{
        const SIZE: usize = mem::size_of::<$typ>();
        // try to convert directly from the bytes
        // this Option<ret> trick is to avoid keeping a borrow on self
        // when advance() is called (mut borrow) and to call bytes() only once
        let ret = $this
            .chunk()
            .get(..SIZE)
            .map(|src| unsafe { $typ::$conv(*(src as *const _ as *const [_; SIZE])) });

        if let Some(ret) = ret {
            // if the direct conversion was possible, advance and return
            $this.advance(SIZE);
            return ret;
        } else {
            // if not we copy the bytes in a temp buffer then convert
            let mut buf = [0; SIZE];
            $this.copy_to_slice(&mut buf); // (do the advance)
            return $typ::$conv(buf);
        }
    }};
    (le => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        // The same trick as above does not improve the best case speed.
        // It seems to be linked to the way the method is optimised by the compiler
        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.copy_to_slice(&mut buf[..($len_to_read)]);
        return $typ::from_le_bytes(buf);
    }};
    (be => $this:ident, $typ:tt, $len_to_read:expr) => {{
        debug_assert!(mem::size_of::<$typ>() >= $len_to_read);

        let mut buf = [0; (mem::size_of::<$typ>())];
        $this.copy_to_slice(&mut buf[mem::size_of::<$typ>() - ($len_to_read)..]);
        return $typ::from_be_bytes(buf);
    }};
}

pub trait MarkBuf: Buf {
    /// 获取从起始值到当前值的slice引用
    fn mark_slice_skip(&mut self, skip: usize) -> &[u8];

    /// 把当前值赋值给起始值
    fn mark_commit(&mut self) -> usize;
    
    /// 更改数据大小
    fn mark_len(&mut self, len: usize);
    
    /// 克隆当前的对象, 并限定范围
    fn mark_clone_slice_range<R: RangeBounds<isize>>(&self, range: R) -> Self where Self: Sized;
    
    /// 克隆当前的对象, 取当前余下的
    fn mark_clone_slice(&self) -> Self where Self: Sized {
        self.mark_clone_slice_range( 0isize .. )
    }

    fn mark_slice(&mut self) -> &[u8] {
        self.mark_slice_skip(0)
    }

    fn mark_bump(&mut self) {
        self.advance(1);
        self.mark_commit();
    }
}

pub trait Buf {
    /// 获取剩余数量
    fn remaining(&self) -> usize;

    /// 获取当前数据的切片引用
    fn chunk(&self) -> &[u8];

    /// 消耗掉多少字节的数据, 做指针偏移
    fn advance(&mut self, n: usize);
    
    /// 消耗所有的字节
    fn advance_all(&mut self) {
        self.advance(self.remaining());
    }

    /// 获取当前的值, 但不做任何偏移
    fn peek(&self) -> Option<u8> {
        if self.has_remaining() {
            let ret = self.chunk()[0] as u8;
            Some(ret)
        } else {
            None
        }
    }

    /// 是否还有数据
    fn has_remaining(&self) -> bool {
        self.remaining() > 0
    }

    /// 获取当前的值并将偏移值+1
    fn get_next(&mut self) -> Option<u8> {
        if self.has_remaining() {
            let val = self.peek().unwrap();
            self.advance(1);
            Some(val)
        } else {
            None
        }
    }

    /// 拷贝数据 `self` into `dst`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary;
    /// use binary::Buf;
    ///
    /// let mut buf = &b"hello world"[..];
    /// let mut dst = [0; 5];
    ///
    /// buf.copy_to_slice(&mut dst);
    /// assert_eq!(&b"hello"[..], &dst);
    /// assert_eq!(6, buf.remaining());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if `self.remaining() < dst.len()`
    fn copy_to_slice(&mut self, dst: &mut [u8]) -> usize {
        assert!(self.remaining() >= dst.len());
        unsafe {
            let src = self.chunk();
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len());
            self.advance(dst.len())
        }
        dst.len()
    }


    fn get_u8(&mut self) -> u8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0];
        self.advance(1);
        ret
    }

    fn get_i8(&mut self) -> i8 {
        assert!(self.remaining() >= 1);
        let ret = self.chunk()[0] as i8;
        self.advance(1);
        ret
    }


    /// Gets an unsigned 16 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x09 hello"[..];
    /// assert_eq!(0x0809, buf.get_u16());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_be_bytes);
    }

    /// Gets an unsigned 16 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x09\x08 hello"[..];
    /// assert_eq!(0x0809, buf.get_u16_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16_le(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_le_bytes);
    }

    /// Gets an unsigned 16 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09 hello",
    ///     false => b"\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809, buf.get_u16_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u16_ne(&mut self) -> u16 {
        buf_get_impl!(self, u16::from_ne_bytes);
    }

    /// Gets a signed 16 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x09 hello"[..];
    /// assert_eq!(0x0809, buf.get_i16());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_be_bytes);
    }

    /// Gets a signed 16 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x09\x08 hello"[..];
    /// assert_eq!(0x0809, buf.get_i16_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16_le(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_le_bytes);
    }

    /// Gets a signed 16 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09 hello",
    ///     false => b"\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809, buf.get_i16_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i16_ne(&mut self) -> i16 {
        buf_get_impl!(self, i16::from_ne_bytes);
    }

    /// Gets an unsigned 32 bit integer from `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x09\xA0\xA1 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_u32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_be_bytes);
    }

    /// Gets an unsigned 32 bit integer from `self` in the little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\xA1\xA0\x09\x08 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_u32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32_le(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_le_bytes);
    }

    /// Gets an unsigned 32 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09\xA0\xA1 hello",
    ///     false => b"\xA1\xA0\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809A0A1, buf.get_u32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u32_ne(&mut self) -> u32 {
        buf_get_impl!(self, u32::from_ne_bytes);
    }

    /// Gets a signed 32 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x09\xA0\xA1 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_i32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_be_bytes);
    }

    /// Gets a signed 32 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\xA1\xA0\x09\x08 hello"[..];
    /// assert_eq!(0x0809A0A1, buf.get_i32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32_le(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_le_bytes);
    }

    /// Gets a signed 32 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x08\x09\xA0\xA1 hello",
    ///     false => b"\xA1\xA0\x09\x08 hello",
    /// };
    /// assert_eq!(0x0809A0A1, buf.get_i32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i32_ne(&mut self) -> i32 {
        buf_get_impl!(self, i32::from_ne_bytes);
    }

    /// Gets an unsigned 64 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_u64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_be_bytes);
    }

    /// Gets an unsigned 64 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_u64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64_le(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_le_bytes);
    }

    /// Gets an unsigned 64 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08 hello",
    ///     false => b"\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x0102030405060708, buf.get_u64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u64_ne(&mut self) -> u64 {
        buf_get_impl!(self, u64::from_ne_bytes);
    }

    /// Gets a signed 64 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_i64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_be_bytes);
    }

    /// Gets a signed 64 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x0102030405060708, buf.get_i64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64_le(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_le_bytes);
    }

    /// Gets a signed 64 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08 hello",
    ///     false => b"\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x0102030405060708, buf.get_i64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i64_ne(&mut self) -> i64 {
        buf_get_impl!(self, i64::from_ne_bytes);
    }

    /// Gets an unsigned 128 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_be_bytes);
    }

    /// Gets an unsigned 128 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128_le(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_le_bytes);
    }

    /// Gets an unsigned 128 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello",
    ///     false => b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_u128_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_u128_ne(&mut self) -> u128 {
        buf_get_impl!(self, u128::from_ne_bytes);
    }

    /// Gets a signed 128 bit integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_be_bytes);
    }

    /// Gets a signed 128 bit integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello"[..];
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128_le(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_le_bytes);
    }

    /// Gets a signed 128 bit integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16 hello",
    ///     false => b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x01020304050607080910111213141516, buf.get_i128_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_i128_ne(&mut self) -> i128 {
        buf_get_impl!(self, i128::from_ne_bytes);
    }

    /// Gets an unsigned n-byte integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03 hello"[..];
    /// assert_eq!(0x010203, buf.get_uint(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint(&mut self, nbytes: usize) -> u64 {
        buf_get_impl!(be => self, u64, nbytes);
    }

    /// Gets an unsigned n-byte integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x03\x02\x01 hello"[..];
    /// assert_eq!(0x010203, buf.get_uint_le(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint_le(&mut self, nbytes: usize) -> u64 {
        buf_get_impl!(le => self, u64, nbytes);
    }

    /// Gets an unsigned n-byte integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03 hello",
    ///     false => b"\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x010203, buf.get_uint_ne(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_uint_ne(&mut self, nbytes: usize) -> u64 {
        if cfg!(target_endian = "big") {
            self.get_uint(nbytes)
        } else {
            self.get_uint_le(nbytes)
        }
    }

    /// Gets a signed n-byte integer from `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x01\x02\x03 hello"[..];
    /// assert_eq!(0x010203, buf.get_int(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int(&mut self, nbytes: usize) -> i64 {
        buf_get_impl!(be => self, i64, nbytes);
    }

    /// Gets a signed n-byte integer from `self` in little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x03\x02\x01 hello"[..];
    /// assert_eq!(0x010203, buf.get_int_le(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int_le(&mut self, nbytes: usize) -> i64 {
        buf_get_impl!(le => self, i64, nbytes);
    }

    /// Gets a signed n-byte integer from `self` in native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x01\x02\x03 hello",
    ///     false => b"\x03\x02\x01 hello",
    /// };
    /// assert_eq!(0x010203, buf.get_int_ne(3));
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_int_ne(&mut self, nbytes: usize) -> i64 {
        if cfg!(target_endian = "big") {
            self.get_int(nbytes)
        } else {
            self.get_int_le(nbytes)
        }
    }

    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x3F\x99\x99\x9A hello"[..];
    /// assert_eq!(1.2f32, buf.get_f32());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32(&mut self) -> f32 {
        f32::from_bits(Self::get_u32(self))
    }

    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x9A\x99\x99\x3F hello"[..];
    /// assert_eq!(1.2f32, buf.get_f32_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32_le(&mut self) -> f32 {
        f32::from_bits(Self::get_u32_le(self))
    }

    /// Gets an IEEE754 single-precision (4 bytes) floating point number from
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x3F\x99\x99\x9A hello",
    ///     false => b"\x9A\x99\x99\x3F hello",
    /// };
    /// assert_eq!(1.2f32, buf.get_f32_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f32_ne(&mut self) -> f32 {
        f32::from_bits(Self::get_u32_ne(self))
    }

    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x3F\xF3\x33\x33\x33\x33\x33\x33 hello"[..];
    /// assert_eq!(1.2f64, buf.get_f64());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64(&mut self) -> f64 {
        f64::from_bits(Self::get_u64(self))
    }

    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf = &b"\x33\x33\x33\x33\x33\x33\xF3\x3F hello"[..];
    /// assert_eq!(1.2f64, buf.get_f64_le());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64_le(&mut self) -> f64 {
        f64::from_bits(Self::get_u64_le(self))
    }

    /// Gets an IEEE754 double-precision (8 bytes) floating point number from
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::Buf;
    ///
    /// let mut buf: &[u8] = match cfg!(target_endian = "big") {
    ///     true => b"\x3F\xF3\x33\x33\x33\x33\x33\x33 hello",
    ///     false => b"\x33\x33\x33\x33\x33\x33\xF3\x3F hello",
    /// };
    /// assert_eq!(1.2f64, buf.get_f64_ne());
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining data in `self`.
    fn get_f64_ne(&mut self) -> f64 {
        f64::from_bits(Self::get_u64_ne(self))
    }
}


impl Buf for &[u8] {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self
    }

    #[inline]
    fn advance(&mut self, cnt: usize) {
        *self = &self[cnt..];
    }

}

impl<T: AsRef<[u8]>> Buf for std::io::Cursor<T> {
    fn remaining(&self) -> usize {
        let len = self.get_ref().as_ref().len();
        let pos = self.position();

        if pos >= len as u64 {
            return 0;
        }

        len - pos as usize
    }

    fn chunk(&self) -> &[u8] {
        let len = self.get_ref().as_ref().len();
        let pos = self.position();

        if pos >= len as u64 {
            return &[];
        }

        &self.get_ref().as_ref()[pos as usize..]
    }

    fn advance(&mut self, cnt: usize) {
        let pos = (self.position() as usize)
            .checked_add(cnt)
            .expect("overflow");

        assert!(pos <= self.get_ref().as_ref().len());
        self.set_position(pos as u64);
    }

}