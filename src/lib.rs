// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;

use std::{arch::asm, rc::Rc};

use crate::task::{RcContext, Task};

// mod runtime;
mod runtime;
mod sys;
pub mod task;

#[test]
fn smoke_test() {
    println!("started");
    let task = Task::new(1 << 15, || {
        println!("while inside");
        debug_registers();
        println!("exiting")
    });

    let cx = RcContext::for_os_thread();
    task.clone().switch(cx.clone());
    unsafe {
        dbg!(cx.0.as_ref().refcount);
    }
    unsafe {
        dbg!(task.0 .0.as_ref().refcount);
    }
    println!("finished");
}
#[inline(always)]
fn debug_registers() {
    let mut x29: u64;
    let mut sp: u64;
    let mut x30: u64;
    unsafe {
        asm!("mov {x30}, x30", "mov {x29}, x29","mov {sp}, sp" ,x29 = out(reg) x29, x30 = out(reg) x30,  sp = out(reg) sp);
    }
    let _sp = sp;
    // dbg!(x29, _sp, x30);
}
// mod utils;

// mod sys;
