#![allow(clippy::new_ret_no_self)]
// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;
extern crate self as pneuma;

// mod runtime;

mod runtime;
pub mod sync;
mod sys;
pub mod thread;

use thread::globals::current;

fn main() {
    let handle = pneuma::thread::spawn(|| {
        dbg!(std::backtrace::Backtrace::force_capture());
        122
    });
    handle.join();
    // pneuma::thread::yield_now();

    // assert_eq!(handle.join(), 122);
}
