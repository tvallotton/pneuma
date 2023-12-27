// use std::{alloc::Layout, mem::zeroed, ptr::null_mut};

// use libc::{mcontext_t, stack_t};

// mod reactor;
// mod runtime;

use std::arch::asm;

use crate::task::Task;

// mod runtime;
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

    task.switch();

    println!("finished");
}
#[inline(always)]
fn debug_registers() {
    let mut x29: u64;
    let mut x30: u64;
    let mut sp: u64;
    unsafe {
        asm!("mov {x30}, x30", "mov {x29}, x29","mov {sp}, sp" ,x29 = out(reg) x29, x30 = out(reg) x30,  sp = out(reg) sp);
    }
    dbg!(x29, x30, sp);
}
// mod utils;

// mod sys;

// #[test]
// fn context_switch() {
//     unsafe { _context_switch() }
// }

// unsafe fn _context_switch() {
//     // let uc_stack = create_stack(1 << 14);

//     // let mut uc_mcontext: mcontext_t = zeroed();

//     // let ucp = libc::ucontext_t {
//     //     uc_flags: 0,
//     //     uc_stack,
//     //     uc_link: null_mut(),
//     //     uc_sigmask: zeroed(),
//     //     uc_mcontext: zeroed(),
//     // };

//     let mut ucp = zeroed();

//     let out = libc::getcontext(&mut ucp);

//     dbg!(out);
//     dbg!(ucp.uc_stack.ss_size);
//     dbg!(ucp.uc_stack.ss_sp);
//     dbg!(ucp.uc_stack.ss_flags);
//     // libc::makecontext(ucp, func, argc);
// }

// use std::{
//     alloc::Layout,
//     arch::{asm, global_asm},
//     mem::zeroed,
//     ptr::{null, null_mut},
// };

// pub struct Context {
//     regs: [u64; 64],
//     stack_alloc: *const libc::c_void,
//     stack_size: usize,
//     fun: Box<dyn FnOnce()>,
// }

// pub enum Data {

// }

// impl Drop for Context {
//     fn drop(&mut self) {
//         unsafe { libc::munmap(self.stack_alloc as _, self.stack_size) };
//     }
// }

// impl Context {

//     fn new(fun: impl FnOnce()) -> Self {

//         let fun = Box::new(fun as _);

//         extern "C" fn

//     }

//     #[rustfmt::skip]
//     fn allocate_new(stack_size: usize, ) -> Self {
//         unsafe {
//             assert!(stack_size > 0);
//             let mut context: Context = zeroed();

//             let mut flags = libc::MAP_ANONYMOUS | libc::MAP_PRIVATE;

//             #[cfg(target_os = "linux")] {
//                 flags |= libc::MAP_STACK;
//             }

//             let alloc = libc::mmap(
//                     null_mut(),
//                     stack_size,
//                     libc::PROT_READ | libc::PROT_WRITE,
//                     libc::MAP_ANONYMOUS,
//                     -1,
//                     0,
//             );

//             assert_ne!(alloc, null_mut(), "allocation failed");
//             context.stack_alloc = alloc;
//             context.stack_size = stack_size;
//             context
//         }
//     }
// }

// std::arch::global_asm!(include_str!("../example.s"));

// extern "C" {
//     pub fn _switch_context(from: *mut mcontext_t, to: *const mcontext_t);
//     pub fn _make_context(
//         ucp: *mut mcontext_t,
//         fun: extern "C" fn(),
//         arg: *mut u8,
//         stack: *mut u8,
//         link: &mut mcontext_t,
//     );
// }

// pub fn switch_context() {}

// pub fn make_context(ucp: &mut ucontext_t, fun: extern "C" fn(), arg: *mut u8) {
//     unsafe {
//         _make_context(
//             &mut ucp.uc_mcontext,
//             fun,
//             arg,
//             &mut ucp.uc_stack.ss_sp as *mut _ as *mut _,
//             &mut (*ucp.uc_link).uc_mcontext,
//         )
//     };
// }

// static mut DONE: bool = false;
// #[test]
// fn test() {
//     let mut main_thread: ucontext_t = unsafe { zeroed() };

//     let mut ss_sp = vec![0u8; 1 << 14];

//     dbg!();
//     unsafe { getcontext(&mut main_thread) };

//     dbg!();
//     if unsafe { DONE } {
//         return;
//     }

//     dbg!();
//     let stack = libc::stack_t {
//         ss_sp: unsafe { ss_sp.as_mut_ptr().offset(ss_sp.len() as _).cast() },
//         ss_size: 1 << 14,
//         ss_flags: 0,
//     };

//     let mut ucp: ucontext_t = unsafe { zeroed() };

//     ucp.uc_stack = stack;
//     ucp.uc_link = &mut main_thread;

//     make_context(&mut ucp, foo, null_mut());

//     dbg!();
//     unsafe { swi(&mut ucp) };
// }

// extern "C" fn foo() {
//     unsafe { DONE = false };

//     println!("hello");
// }
