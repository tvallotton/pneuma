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

use std::thread::yield_now;

use thread::globals::current;
#[inline(never)]
fn bar() {
    // unsafe { std::arch::asm!("mov x30, #0") }
    // unsafe { std::arch::asm!("mov x29, #0") }
    // unsafe { std::arch::asm!("mov x28, #0") }
    // unsafe { std::arch::asm!("mov x27, #0") }
    // unsafe { std::arch::asm!("mov x26, #0") }
    // unsafe { std::arch::asm!("mov x25, #0") }
    // unsafe { std::arch::asm!("mov x24, #0") }
    // unsafe { std::arch::asm!("mov x23, #0") }
    // unsafe { std::arch::asm!("mov x22, #0") }
    // unsafe { std::arch::asm!("mov x21, #0") }
    // unsafe { std::arch::asm!("mov x20, #0") }
    // unsafe { std::arch::asm!("mov x19, #0") }
    // unsafe { std::arch::asm!("mov x18, #0") }
    // unsafe { std::arch::asm!("mov x17, #0") }
    // unsafe { std::arch::asm!("mov x16, #0") }
    // unsafe { std::arch::asm!("mov x15, #0") }
    // unsafe { std::arch::asm!("mov x14, #0") }
    // unsafe { std::arch::asm!("mov x13, #0") }
    // unsafe { std::arch::asm!("mov x12, #0") }
    // unsafe { std::arch::asm!("mov x11, #0") }
    // unsafe { std::arch::asm!("mov x10, #0") }
    // unsafe { std::arch::asm!("mov x9, #0") }
    // unsafe { std::arch::asm!("mov x8, #0") }
    // unsafe { std::arch::asm!("mov x7, #0") }
    // unsafe { std::arch::asm!("mov x6, #0") }
    // unsafe { std::arch::asm!("mov x5, #0") }
    // unsafe { std::arch::asm!("mov x4, #0") }
    // unsafe { std::arch::asm!("mov x3, #0") }
    // unsafe { std::arch::asm!("mov x2, #0") }
    // unsafe { std::arch::asm!("mov x1, #0") }
    // unsafe { std::arch::asm!("mov x0, #0") }

    // unsafe { std::arch::asm!(".cfi_undefined x30") }
    // unsafe { std::arch::asm!(".cfi_undefined x29") }
    // unsafe { std::arch::asm!(".cfi_undefined x28") }
    // unsafe { std::arch::asm!(".cfi_undefined x27") }
    // unsafe { std::arch::asm!(".cfi_undefined x26") }
    // unsafe { std::arch::asm!(".cfi_undefined x25") }
    // unsafe { std::arch::asm!(".cfi_undefined x24") }
    // unsafe { std::arch::asm!(".cfi_undefined x23") }
    // unsafe { std::arch::asm!(".cfi_undefined x22") }
    // unsafe { std::arch::asm!(".cfi_undefined x21") }
    // unsafe { std::arch::asm!(".cfi_undefined x20") }
    // unsafe { std::arch::asm!(".cfi_undefined x19") }
    // unsafe { std::arch::asm!(".cfi_undefined x18") }
    // unsafe { std::arch::asm!(".cfi_undefined x17") }
    // unsafe { std::arch::asm!(".cfi_undefined x16") }
    // unsafe { std::arch::asm!(".cfi_undefined x15") }
    // unsafe { std::arch::asm!(".cfi_undefined x14") }
    // unsafe { std::arch::asm!(".cfi_undefined x13") }
    // unsafe { std::arch::asm!(".cfi_undefined x12") }
    // unsafe { std::arch::asm!(".cfi_undefined x11") }
    // unsafe { std::arch::asm!(".cfi_undefined x10") }
    // unsafe { std::arch::asm!(".cfi_undefined x9") }
    // unsafe { std::arch::asm!(".cfi_undefined x8") }
    // unsafe { std::arch::asm!(".cfi_undefined x7") }
    // unsafe { std::arch::asm!(".cfi_undefined x6") }
    // unsafe { std::arch::asm!(".cfi_undefined x5") }
    // unsafe { std::arch::asm!(".cfi_undefined x4") }
    // unsafe { std::arch::asm!(".cfi_undefined x3") }
    // unsafe { std::arch::asm!(".cfi_undefined x2") }
    // unsafe { std::arch::asm!(".cfi_undefined x1") }
    // unsafe { std::arch::asm!(".cfi_undefined x0") }

    yield_now();
    println!("add");
    println!("{}", std::backtrace::Backtrace::force_capture());
}

fn main() {
    // assert!(!std::backtrace::Backtrace::force_capture()
    //     .to_string()
    //     .is_empty());
    // bar();
    let handle = pneuma::thread::spawn(|| {
        println!("asd");
        for i in 0..1000 {
            yield_now();
        }

        bar();
        122
    });
    // println!("{}", std::backtrace::Backtrace::capture());
    for i in 0..10 {
        yield_now();
    }
    handle.join();

    // pneuma::thread::yield_now();

    // assert_eq!(handle.join(), 122);
}
