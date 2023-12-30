#![allow(clippy::new_ret_no_self)]
// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;
extern crate self as pneuma;

use std::{arch::asm, hint::black_box, rc::Rc, time::UNIX_EPOCH};

use pneuma::thread::RcContext;

// mod runtime;
mod runtime;
mod sys;
pub mod thread;

pub use thread::globals::current;

use pneuma::thread::{spawn, Builder};

#[test]
fn smoke_test() {
    println!("started");

    // let cx = pneuma::current();

    let handle = pneuma::spawn(|| {
        println!("while inside");

        println!("exiting");
    });
    drop(handle);
    // pneuma::thread::park();

    println!("finished");
}
