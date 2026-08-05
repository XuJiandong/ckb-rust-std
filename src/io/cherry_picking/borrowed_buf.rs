use alloc::fmt::{self, Debug, Formatter};
use core::mem::{self, MaybeUninit};
use core::ptr;

pub struct BorrowedBuf<'data> {
    buf: &'data mut [MaybeUninit<u8>],
    filled: usize,
    init: bool,
}

impl Debug for BorrowedBuf<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedBuf")
            .field("init", &self.init)
            .field("filled", &self.filled)
            .field("capacity", &self.capacity())
            .finish()
    }
}

impl<'data> From<&'data mut [u8]> for BorrowedBuf<'data> {
    #[inline]
    fn from(slice: &'data mut [u8]) -> BorrowedBuf<'data> {
        BorrowedBuf {
            // SAFETY: initialized data never becoming uninitialized is an invariant of BorrowedBuf
            buf: unsafe { &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>]) },
            filled: 0,
            init: true,
        }
    }
}

impl<'data> From<&'data mut [MaybeUninit<u8>]> for BorrowedBuf<'data> {
    #[inline]
    fn from(buf: &'data mut [MaybeUninit<u8>]) -> BorrowedBuf<'data> {
        BorrowedBuf {
            buf,
            filled: 0,
            init: false,
        }
    }
}

impl<'data> From<BorrowedCursor<'data>> for BorrowedBuf<'data> {
    #[inline]
    fn from(buf: BorrowedCursor<'data>) -> BorrowedBuf<'data> {
        BorrowedBuf {
            // SAFETY: no initialized byte is ever uninitialized as per
            // `BorrowedBuf`'s invariant
            buf: unsafe { buf.buf.buf.get_unchecked_mut(buf.buf.filled..) },
            filled: 0,
            init: buf.buf.init,
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<'data> BorrowedBuf<'data> {
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.init
    }

    #[inline]
    pub fn filled(&self) -> &[u8] {
        // SAFETY: We only slice the filled part of the buffer, which is always valid
        unsafe {
            let buf = self.buf.get_unchecked(..self.filled);
            &*(buf as *const [MaybeUninit<u8>] as *const [u8])
        }
    }

    #[inline]
    pub fn filled_mut(&mut self) -> &mut [u8] {
        // SAFETY: We only slice the filled part of the buffer, which is always valid
        unsafe {
            let buf = self.buf.get_unchecked_mut(..self.filled);
            &mut *(buf as *mut [MaybeUninit<u8>] as *mut [u8])
        }
    }

    #[inline]
    pub fn into_filled(self) -> &'data [u8] {
        // SAFETY: We only slice the filled part of the buffer, which is always valid
        unsafe {
            let buf = self.buf.get_unchecked(..self.filled);
            &*(buf as *const [MaybeUninit<u8>] as *const [u8])
        }
    }

    #[inline]
    pub fn into_filled_mut(self) -> &'data mut [u8] {
        // SAFETY: We only slice the filled part of the buffer, which is always valid
        unsafe {
            let buf = self.buf.get_unchecked_mut(..self.filled);
            &mut *(buf as *mut [MaybeUninit<u8>] as *mut [u8])
        }
    }

    #[inline]
    pub fn unfilled<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor {
            // SAFETY: we never assign into `BorrowedCursor::buf`, so treating its
            // lifetime covariantly is safe.
            buf: unsafe {
                mem::transmute::<&'this mut BorrowedBuf<'data>, &'this mut BorrowedBuf<'this>>(self)
            },
        }
    }

    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.filled = 0;
        self
    }

    /// # Safety
    ///
    /// Must only be called if all bytes in the buffer are initialized.
    #[inline]
    pub unsafe fn set_init(&mut self) -> &mut Self {
        self.init = true;
        self
    }
}

#[derive(Debug)]
pub struct BorrowedCursor<'a> {
    // Safety invariant: we treat the type of buf as covariant in the lifetime of `BorrowedBuf` when
    // we create a `BorrowedCursor`. This is only safe if we never replace `buf` by assigning into
    // it, so don't do that!
    buf: &'a mut BorrowedBuf<'a>,
}

impl<'a> BorrowedCursor<'a> {
    #[inline]
    pub fn reborrow<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor {
            // SAFETY: we never assign into `BorrowedCursor::buf`, so treating its
            // lifetime covariantly is safe.
            buf: unsafe {
                mem::transmute::<&'this mut BorrowedBuf<'a>, &'this mut BorrowedBuf<'this>>(
                    self.buf,
                )
            },
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity() - self.buf.filled
    }

    #[inline]
    pub fn written(&self) -> usize {
        self.buf.filled
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.buf.init
    }

    /// # Safety
    ///
    /// Must only be called if all bytes in the unfilled part of the buffer
    /// have been initialized.
    #[inline]
    pub unsafe fn set_init(&mut self) {
        self.buf.init = true;
    }

    /// # Safety
    ///
    /// Must only be called if the returned bytes will be initialized before
    /// being read.
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        // SAFETY: always in bounds
        unsafe { self.buf.buf.get_unchecked_mut(self.buf.filled..) }
    }

    #[inline]
    pub fn advance_checked(&mut self, n: usize) -> &mut Self {
        // The subtraction cannot underflow by invariant of this type.
        let init_unfilled = if self.buf.init {
            self.buf.buf.len() - self.buf.filled
        } else {
            0
        };
        assert!(n <= init_unfilled);

        self.buf.filled += n;
        self
    }

    /// # Safety
    ///
    /// The caller must ensure that `n` bytes have been initialized and are
    /// safe to be exposed as filled.
    #[inline]
    pub unsafe fn advance(&mut self, n: usize) -> &mut Self {
        self.buf.filled += n;
        self
    }

    #[inline]
    pub fn ensure_init(&mut self) -> &mut [u8] {
        // SAFETY: always in bounds and we never uninitialize these bytes.
        let unfilled = unsafe { self.buf.buf.get_unchecked_mut(self.buf.filled..) };

        if !self.buf.init {
            // SAFETY: 0 is a valid value for MaybeUninit<u8> and the length matches the allocation
            // since it is comes from a slice reference.
            unsafe {
                ptr::write_bytes(unfilled.as_mut_ptr(), 0, unfilled.len());
            }
            self.buf.init = true;
        }

        // SAFETY: these bytes have just been initialized if they weren't before
        unsafe { &mut *(unfilled as *mut [MaybeUninit<u8>] as *mut [u8]) }
    }

    #[inline]
    pub fn append(&mut self, buf: &[u8]) {
        assert!(self.capacity() >= buf.len());

        // SAFETY: we do not de-initialize any of the elements of the slice
        unsafe {
            let dst = self.as_mut()[..buf.len()].as_mut_ptr() as *mut u8;
            ptr::copy_nonoverlapping(buf.as_ptr(), dst, buf.len());
        }

        self.buf.filled += buf.len();
    }

    pub fn with_unfilled_buf<T>(&mut self, f: impl FnOnce(&mut BorrowedBuf<'_>) -> T) -> T {
        let mut buf = BorrowedBuf::from(self.reborrow());
        let prev_ptr = buf.buf as *const _;
        let res = f(&mut buf);

        // Check that the caller didn't replace the `BorrowedBuf`.
        // This is necessary for the safety of the code below: if the check wasn't
        // there, one could mark some bytes as initialized even though there aren't.
        assert!(core::ptr::eq(prev_ptr, buf.buf));

        let filled = buf.filled;
        let init = buf.init;

        // Update `init` and `filled` fields with what was written to the buffer.
        // `self.buf.filled` was the starting length of the `BorrowedBuf`.
        //
        // SAFETY: These amounts of bytes were initialized/filled in the `BorrowedBuf`,
        // and therefore they are initialized/filled in the cursor too, because the
        // buffer wasn't replaced.
        self.buf.init = init;
        self.buf.filled += filled;

        res
    }
}
