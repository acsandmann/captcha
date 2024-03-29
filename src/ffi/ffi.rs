use crate::ffi::mem;
static mut token: u64 = 0;

#[link(wasm_import_module = "io")]
extern "C" {
  pub fn drop(id: u64);
  pub fn write(id: u64, ptr: mem::buf);
  pub fn read(id: u64, ptr: mem::buf) -> usize;
}

#[no_mangle]
pub unsafe extern "C" fn wlen() -> usize { mem::length() }
#[no_mangle]
pub unsafe extern "C" fn wtoken() -> u64 {
  token += 1;
  token - 1
}
#[no_mangle]
pub unsafe extern "C" fn wfree(ptr: mem::buf, size: usize) { mem::free(ptr, size); }
#[no_mangle]
pub unsafe extern "C" fn walloc(size: usize) -> mem::buf { mem::alloc(size) }
