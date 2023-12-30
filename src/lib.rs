// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;

use std::{arch::asm, hint::black_box, rc::Rc, time::UNIX_EPOCH};

use crate::task::{RcContext, Task};

// mod runtime;
pub mod runtime;
mod sys;
pub mod task;

#[test]
fn smoke_test() {
    println!("started");
    let task = Task::new(1 << 14, || {
        println!("while inside");

        println!("exiting")
    })
    .unwrap();

    let cx = RcContext::for_os_thread();
    task.clone().switch(cx.clone());

    println!("finished");
}
