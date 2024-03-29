use alloc::{boxed::Box, string::String};

#[inline]
pub fn drop<T>(ptr: *mut T) { unpack(ptr); }
#[inline(always)]
pub const fn err<T>(code: u8) -> *mut T { code as *mut T }
#[inline]
pub fn pack<T>(value: T) -> *mut T { Box::into_raw(Box::new(value)) }
#[inline]
pub fn unpack<T>(ptr: *mut T) -> Box<T> { unsafe { Box::from_raw(ptr) } }
#[inline]
pub unsafe fn str(s: &str) -> crate::ffi::mem::buf { core::mem::transmute(s.as_bytes().as_ptr()) }

#[inline]
pub unsafe fn string(s: String) -> crate::ffi::mem::buf { core::mem::transmute(s.into_bytes().as_ptr()) }

#[macro_export]
macro_rules! define_generic_destructors {
    ($(($name:ident => $type:ty)),+) => { $(#[no_mangle] unsafe extern "C" fn $name(ptr: $type) { alloc::boxed::Box::from_raw(ptr); })+ };
}
