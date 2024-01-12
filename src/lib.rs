#![allow(clippy::fn_to_numeric_cast)]
#![allow(clippy::new_ret_no_self)]
// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;
extern crate self as pneuma;

// mod runtime;
#[cfg(feature = "io")]
mod reactor;
mod runtime;
pub mod sync;
mod sys;
pub mod thread;
#[cfg(feature = "io")]
pub mod time;
mod utils;
