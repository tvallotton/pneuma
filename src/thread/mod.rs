use std::{
    cell::UnsafeCell, io::Error, marker::PhantomData, mem::zeroed, os::raw::c_void, ptr::null_mut,
};

use crate::syscall;

mod context;
mod join_handle;

pub struct Thread {
    context: libc::ucontext_t,
}

impl Thread {
    fn new(f: extern "C" fn(), stack_size: usize) -> Self {
        unsafe { Self::_new(f, stack_size) }
    }
    unsafe fn _new(f: extern "C" fn(), stack_size: usize) -> Self {
        let stack = stack(stack_size);
        todo!()
        // let mut context = { zeroed() };
        // context.uc_link:
        // Thread {
        //     context: libc::ucontext_t {
        //         uc_flags: (),
        //         uc_link: (),
        //         uc_stack: (),
        //         uc_sigmask: (),
        //         uc_mcontext: (),
        //     },
        // }
    }
}

fn stack(ss_size: usize) -> libc::stack_t {
    assert!(ss_size != 0, "stack size can't be zero.");

    let map_flags = libc::MAP_ANON; //| libc::MAP_STACK;
    let prot = libc::PROT_READ | libc::PROT_WRITE;

    let ss_sp: *mut c_void =
        unsafe { libc::mmap(null_mut(), ss_size, prot, map_flags, -1, 0).cast() };

    assert!(!ss_sp.is_null());
    libc::stack_t {
        ss_sp,
        ss_size,
        ss_flags: 0,
    }
}
pub struct JoinHandle<'a, T> {
    task: Task,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> JoinHandle<'a, T> {
    pub fn new(task: Task) -> Self {
        JoinHandle {
            task,
            _marker: PhantomData,
        }
    }
}

pub struct Task {
    context: libc::ucontext_t,
}

pub enum TaskState {
    Pending,
}
