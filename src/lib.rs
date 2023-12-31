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

use crate::thread::yield_now;

#[test]
fn smoke_test() {
    let handle = pneuma::thread::spawn(|| {
        println!("task: init");
        yield_now();
        println!("task: middle ");
        yield_now();
        println!("task: exiting");
        122
    });

    println!("main: spawned");
    yield_now();
    println!("main: yielded");
    assert_eq!(handle.join(), 122);
    println!("main: finished");
}
