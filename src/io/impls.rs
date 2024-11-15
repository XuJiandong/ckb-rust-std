use crate::io::{self, BorrowedCursor, BufRead, Read, Seek, SeekFrom, Write};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::fmt;
use alloc::str;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::mem;
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }
    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }
}
impl<B: BufRead + ?Sized> BufRead for &mut B {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }
}
impl<B: BufRead + ?Sized> BufRead for Box<B> {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        (**self).fill_buf()
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }

    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
        (**self).read_until(byte, buf)
    }

    #[inline]
    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        (**self).read_line(buf)
    }
}

// =============================================================================
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }

    #[inline]
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let amt = cmp::min(cursor.capacity(), self.len());
        let (a, b) = self.split_at(amt);

        cursor.append(a);

        *self = b;
        Ok(())
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            // `read_exact` makes no promise about the content of `buf` if it
            // fails so don't bother about that.
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() > self.len() {
            // Append everything we can to the cursor.
            cursor.append(self);
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(cursor.capacity());

        cursor.append(a);

        *self = b;
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(self);
        *self = &self[len..];
        Ok(len)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        let content = str::from_utf8(self).map_err(|_| io::Error::INVALID_UTF8)?;
        let len = self.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        *self = &self[len..];
        Ok(len)
    }
}
impl BufRead for &[u8] {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(*self)
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? == data.len() {
            Ok(())
        } else {
            Err(crate::io::Error::WRITE_ALL_EOF)
        }
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
impl Write for Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
impl Read for VecDeque<u8> {
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }

    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        let (ref mut front, _) = self.as_slices();
        let n = cmp::min(cursor.capacity(), front.len());
        Read::read_buf(front, cursor)?;
        self.drain(..n);
        Ok(())
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        // The total len is known upfront so we can reserve it in a single call.
        let len = self.len();
        buf.try_reserve(len)?;

        let (front, back) = self.as_slices();
        buf.extend_from_slice(front);
        buf.extend_from_slice(back);
        self.clear();
        Ok(len)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        // SAFETY: We only append to the buffer
        unsafe { io::append_to_string(buf, |buf| self.read_to_end(buf)) }
    }
}
/// BufRead is implemented for `VecDeque<u8>` by reading bytes from the front of the `VecDeque`.
impl BufRead for VecDeque<u8> {
    /// Returns the contents of the "front" slice as returned by
    /// [`as_slices`][`VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `fill_buf` will be needed to read the entire content.
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let (front, _) = self.as_slices();
        Ok(front)
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.drain(..amt);
    }
}

/// Write is implemented for `VecDeque<u8>` by appending to the `VecDeque`, growing it as needed.
impl Write for VecDeque<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl<'a> io::Write for io::BorrowedCursor<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.capacity());
        self.append(&buf[..amt]);
        Ok(amt)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
