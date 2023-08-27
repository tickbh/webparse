use std::{ptr, cmp, mem::MaybeUninit};


pub unsafe trait BufMut {
    
    fn remaining_mut(&self) -> usize;
    unsafe fn advance_mut(&mut self, cnt: usize);
    fn chunk_mut(&mut self) -> &mut [MaybeUninit<u8>];

    fn has_remaining_mut(&self) -> bool {
        self.remaining_mut() > 0
    }


    fn put<T: super::Buf>(&mut self, mut src: T)
    where
        Self: Sized,
    {
        assert!(self.remaining_mut() >= src.remaining());

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
    }

}