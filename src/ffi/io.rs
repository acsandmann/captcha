use alloc::{string::String, vec::Vec};

use crate::ffi::mem;
#[cfg(feature = "fd")]
pub struct fd {
  pub id: u64,
}
#[inline]
pub fn load(ptr: mem::buf, size: usize) -> Vec<u8> { unsafe { Vec::from_raw_parts(ptr, size, size) } }
#[inline]
pub fn peek(buffer: &[u8]) -> mem::buf {
  unsafe {
    mem::len = buffer.len();
  }
  buffer.as_ptr() as mem::buf
}
#[inline]
pub fn string(ptr: mem::buf, size: usize) -> String { unsafe { String::from_utf8_unchecked(load(ptr, size)) } }
#[inline]
pub fn str<'a>(ptr: mem::buf, size: usize) -> &'a str { unsafe { core::mem::transmute(&*core::ptr::slice_from_raw_parts(ptr, size)) } }
#[inline]
pub fn store(mut buf: Vec<u8>) -> mem::buf {
  buf.shrink_to_fit();
  unsafe {
    mem::len = buf.len();
  }
  let ptr = buf.as_mut_ptr();
  core::mem::forget(buf);
  ptr
}

#[cfg(feature = "fd")]
impl fd {
  pub fn new(id: u64) -> Self { fd { id } }
}

#[cfg(feature = "fd")]
impl Drop for fd {
  fn drop(&mut self) { unsafe { ffi::drop(self.id) }; }
}

#[cfg(feature = "fd")]
impl io::Read for fd {
  fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> { unsafe { Ok(ffi::read(self.id, peek(buffer))) } }
}

#[cfg(feature = "fd")]
impl io::Write for fd {
  fn flush(&mut self) -> io::Result<()> { Ok(()) }

  fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
    unsafe { ffi::write(self.id, peek(buffer)) };
    Ok(buffer.len())
  }
}
