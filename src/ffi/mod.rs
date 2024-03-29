#![allow(dead_code)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(mutable_transmutes)]
#![allow(clippy::forget_ref)]
#![cfg_attr(feature = "coerce", feature(unsize, coerce_unsized))]

extern crate alloc;

pub type void = core::ffi::c_void;
pub type any = *mut ();

pub mod ffi;
pub mod io;
pub mod mem;
pub mod ptr;
