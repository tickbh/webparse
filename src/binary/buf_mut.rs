use std::{
    cmp,
    mem::{self, MaybeUninit},
    ptr,
};

pub unsafe trait BufMut {
    fn remaining_mut(&self) -> usize;
    unsafe fn advance_mut(&mut self, cnt: usize);
    fn chunk_mut(&mut self) -> &mut [MaybeUninit<u8>];

    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }

    fn put<T: super::Buf>(&mut self, src: &mut T) -> usize
    where
        Self: Sized,
    {
        assert!(self.remaining_mut() >= src.remaining());
        let len = src.remaining();
        while src.has_remaining() {
            let l;

            unsafe {
                let s = src.chunk();
                let d = self.chunk_mut();
                l = cmp::min(s.len(), d.len());

                ptr::copy_nonoverlapping(s.as_ptr(), d.as_mut_ptr() as *mut u8, l);
            }

            src.advance(l);
            unsafe {
                self.advance_mut(l);
            }
        }
        len
    }

    fn put_slice(&mut self, src: &[u8]) -> usize {
        let mut off = 0;

        assert!(
            self.remaining_mut() >= src.len(),
            "buffer overflow; remaining = {}; src = {}",
            self.remaining_mut(),
            src.len()
        );

        while off < src.len() {
            let cnt;

            unsafe {
                let dst = self.chunk_mut();
                cnt = cmp::min(dst.len(), src.len() - off);

                ptr::copy_nonoverlapping(src[off..].as_ptr(), dst.as_mut_ptr() as *mut u8, cnt);

                off += cnt;
            }

            unsafe {
                self.advance_mut(cnt);
            }
        }
        src.len()
    }

    fn put_bytes(&mut self, val: u8, cnt: usize) {
        for _ in 0..cnt {
            self.put_u8(val);
        }
    }

    fn put_u8(&mut self, n: u8) {
        let src = [n];
        self.put_slice(&src);
    }

    fn put_i8(&mut self, n: i8) {
        let src = [n as u8];
        self.put_slice(&src);
    }

    /// Writes an unsigned 16 bit integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u16(0x0809);
    /// assert_eq!(buf, b"\x08\x09");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u16(&mut self, n: u16) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes an unsigned 16 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u16_le(0x0809);
    /// assert_eq!(buf, b"\x09\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u16_le(&mut self, n: u16) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes an unsigned 16 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u16_ne(0x0809);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x08\x09");
    /// } else {
    ///     assert_eq!(buf, b"\x09\x08");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u16_ne(&mut self, n: u16) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes a signed 16 bit integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i16(0x0809);
    /// assert_eq!(buf, b"\x08\x09");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i16(&mut self, n: i16) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes a signed 16 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i16_le(0x0809);
    /// assert_eq!(buf, b"\x09\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i16_le(&mut self, n: i16) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes a signed 16 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i16_ne(0x0809);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x08\x09");
    /// } else {
    ///     assert_eq!(buf, b"\x09\x08");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i16_ne(&mut self, n: i16) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes an unsigned 32 bit integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u32(0x0809A0A1);
    /// assert_eq!(buf, b"\x08\x09\xA0\xA1");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u32(&mut self, n: u32) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes an unsigned 32 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u32_le(0x0809A0A1);
    /// assert_eq!(buf, b"\xA1\xA0\x09\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u32_le(&mut self, n: u32) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes an unsigned 32 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u32_ne(0x0809A0A1);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x08\x09\xA0\xA1");
    /// } else {
    ///     assert_eq!(buf, b"\xA1\xA0\x09\x08");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u32_ne(&mut self, n: u32) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes a signed 32 bit integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i32(0x0809A0A1);
    /// assert_eq!(buf, b"\x08\x09\xA0\xA1");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i32(&mut self, n: i32) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes a signed 32 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i32_le(0x0809A0A1);
    /// assert_eq!(buf, b"\xA1\xA0\x09\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i32_le(&mut self, n: i32) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes a signed 32 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i32_ne(0x0809A0A1);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x08\x09\xA0\xA1");
    /// } else {
    ///     assert_eq!(buf, b"\xA1\xA0\x09\x08");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i32_ne(&mut self, n: i32) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes an unsigned 64 bit integer to `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u64(0x0102030405060708);
    /// assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u64(&mut self, n: u64) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes an unsigned 64 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u64_le(0x0102030405060708);
    /// assert_eq!(buf, b"\x08\x07\x06\x05\x04\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u64_le(&mut self, n: u64) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes an unsigned 64 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u64_ne(0x0102030405060708);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08");
    /// } else {
    ///     assert_eq!(buf, b"\x08\x07\x06\x05\x04\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u64_ne(&mut self, n: u64) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes a signed 64 bit integer to `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i64(0x0102030405060708);
    /// assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i64(&mut self, n: i64) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes a signed 64 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i64_le(0x0102030405060708);
    /// assert_eq!(buf, b"\x08\x07\x06\x05\x04\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i64_le(&mut self, n: i64) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes a signed 64 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i64_ne(0x0102030405060708);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08");
    /// } else {
    ///     assert_eq!(buf, b"\x08\x07\x06\x05\x04\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i64_ne(&mut self, n: i64) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes an unsigned 128 bit integer to `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u128(0x01020304050607080910111213141516);
    /// assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u128(&mut self, n: u128) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes an unsigned 128 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u128_le(0x01020304050607080910111213141516);
    /// assert_eq!(buf, b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u128_le(&mut self, n: u128) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes an unsigned 128 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_u128_ne(0x01020304050607080910111213141516);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16");
    /// } else {
    ///     assert_eq!(buf, b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_u128_ne(&mut self, n: u128) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes a signed 128 bit integer to `self` in the big-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i128(0x01020304050607080910111213141516);
    /// assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i128(&mut self, n: i128) {
        self.put_slice(&n.to_be_bytes());
    }

    /// Writes a signed 128 bit integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i128_le(0x01020304050607080910111213141516);
    /// assert_eq!(buf, b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i128_le(&mut self, n: i128) {
        self.put_slice(&n.to_le_bytes());
    }

    /// Writes a signed 128 bit integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by 16.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_i128_ne(0x01020304050607080910111213141516);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16");
    /// } else {
    ///     assert_eq!(buf, b"\x16\x15\x14\x13\x12\x11\x10\x09\x08\x07\x06\x05\x04\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_i128_ne(&mut self, n: i128) {
        self.put_slice(&n.to_ne_bytes());
    }

    /// Writes an unsigned n-byte integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_uint(0x010203, 3);
    /// assert_eq!(buf, b"\x01\x02\x03");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_uint(&mut self, n: u64, nbytes: usize) {
        self.put_slice(&n.to_be_bytes()[mem::size_of_val(&n) - nbytes..]);
    }

    /// Writes an unsigned n-byte integer to `self` in the little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_uint_le(0x010203, 3);
    /// assert_eq!(buf, b"\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_uint_le(&mut self, n: u64, nbytes: usize) {
        self.put_slice(&n.to_le_bytes()[0..nbytes]);
    }

    /// Writes an unsigned n-byte integer to `self` in the native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_uint_ne(0x010203, 3);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03");
    /// } else {
    ///     assert_eq!(buf, b"\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_uint_ne(&mut self, n: u64, nbytes: usize) {
        if cfg!(target_endian = "big") {
            self.put_uint(n, nbytes)
        } else {
            self.put_uint_le(n, nbytes)
        }
    }

    /// Writes low `nbytes` of a signed integer to `self` in big-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_int(0x0504010203, 3);
    /// assert_eq!(buf, b"\x01\x02\x03");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self` or if `nbytes` is greater than 8.
    fn put_int(&mut self, n: i64, nbytes: usize) {
        self.put_slice(&n.to_be_bytes()[mem::size_of_val(&n) - nbytes..]);
    }

    /// Writes low `nbytes` of a signed integer to `self` in little-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_int_le(0x0504010203, 3);
    /// assert_eq!(buf, b"\x03\x02\x01");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self` or if `nbytes` is greater than 8.
    fn put_int_le(&mut self, n: i64, nbytes: usize) {
        self.put_slice(&n.to_le_bytes()[0..nbytes]);
    }

    /// Writes low `nbytes` of a signed integer to `self` in native-endian byte order.
    ///
    /// The current position is advanced by `nbytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_int_ne(0x010203, 3);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x01\x02\x03");
    /// } else {
    ///     assert_eq!(buf, b"\x03\x02\x01");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self` or if `nbytes` is greater than 8.
    fn put_int_ne(&mut self, n: i64, nbytes: usize) {
        if cfg!(target_endian = "big") {
            self.put_int(n, nbytes)
        } else {
            self.put_int_le(n, nbytes)
        }
    }

    /// Writes  an IEEE754 single-precision (4 bytes) floating point number to
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f32(1.2f32);
    /// assert_eq!(buf, b"\x3F\x99\x99\x9A");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f32(&mut self, n: f32) {
        self.put_u32(n.to_bits());
    }

    /// Writes  an IEEE754 single-precision (4 bytes) floating point number to
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f32_le(1.2f32);
    /// assert_eq!(buf, b"\x9A\x99\x99\x3F");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f32_le(&mut self, n: f32) {
        self.put_u32_le(n.to_bits());
    }

    /// Writes an IEEE754 single-precision (4 bytes) floating point number to
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 4.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f32_ne(1.2f32);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x3F\x99\x99\x9A");
    /// } else {
    ///     assert_eq!(buf, b"\x9A\x99\x99\x3F");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f32_ne(&mut self, n: f32) {
        self.put_u32_ne(n.to_bits());
    }

    /// Writes  an IEEE754 double-precision (8 bytes) floating point number to
    /// `self` in big-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f64(1.2f64);
    /// assert_eq!(buf, b"\x3F\xF3\x33\x33\x33\x33\x33\x33");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f64(&mut self, n: f64) {
        self.put_u64(n.to_bits());
    }

    /// Writes  an IEEE754 double-precision (8 bytes) floating point number to
    /// `self` in little-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f64_le(1.2f64);
    /// assert_eq!(buf, b"\x33\x33\x33\x33\x33\x33\xF3\x3F");
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f64_le(&mut self, n: f64) {
        self.put_u64_le(n.to_bits());
    }

    /// Writes  an IEEE754 double-precision (8 bytes) floating point number to
    /// `self` in native-endian byte order.
    ///
    /// The current position is advanced by 8.
    ///
    /// # Examples
    ///
    /// ```
    /// use webparse::binary::BufMut;
    ///
    /// let mut buf = vec![];
    /// buf.put_f64_ne(1.2f64);
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(buf, b"\x3F\xF3\x33\x33\x33\x33\x33\x33");
    /// } else {
    ///     assert_eq!(buf, b"\x33\x33\x33\x33\x33\x33\xF3\x3F");
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This function panics if there is not enough remaining capacity in
    /// `self`.
    fn put_f64_ne(&mut self, n: f64) {
        self.put_u64_ne(n.to_bits());
    }
}
