#![allow(clippy::new_ret_no_self)]
// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;
extern crate self as pneuma;

use std::{arch::asm, hint::black_box, rc::Rc, time::UNIX_EPOCH};

use pneuma::task::{RcContext, Task};

// mod runtime;
mod runtime;
mod sys;
pub mod task;

pub use task::globals::{current, os_thread, try_green_thread, GREEN_THREAD};

use pneuma::task::{spawn, Builder};

#[test]
fn smoke_test() {
    println!("started");

    let cx = pneuma::current();

    let handle = Task::new(
        || {
            println!("while inside");

            println!("exiting");
        },
        Builder::new(),
    )
    .unwrap();

    println!("foo");
    handle.switch(cx);

    println!("finished");
}
