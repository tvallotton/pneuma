#![allow(clippy::fn_to_numeric_cast)]
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

pub use thread::globals::current;

#[test]
fn smoke_test() {
    let handle = pneuma::thread::spawn(|| {
        println!("task: init");
        pneuma::thread::yield_now();
        println!("task: middle ");
        pneuma::thread::yield_now();
        println!("task: exiting");
        122
    });

    println!("main: spawned");
    pneuma::thread::yield_now();
    println!("main: yielded");
    assert_eq!(handle.join(), 122);
    println!("main: finished");
}
#[test]
fn foo() {}
