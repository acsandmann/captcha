#![allow(non_camel_case_types)]
pub type buf = *mut u8;
pub static mut len: usize = 0;

#[inline]
pub fn length() -> usize { unsafe { len } }

#[inline]
pub fn alloc(size: usize) -> buf { unsafe { alloc::alloc::alloc(alloc::alloc::Layout::from_size_align_unchecked(size, 1)) } }

#[inline]
pub fn free(ptr: buf, size: usize) { unsafe { alloc::alloc::dealloc(ptr, alloc::alloc::Layout::from_size_align_unchecked(size, 1)) } }
