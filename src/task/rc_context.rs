use std::{
    arch::asm,
    backtrace::{self, Backtrace},
    cell::{Cell, UnsafeCell},
    ptr::NonNull,
};

use crate::{debug_registers, sys};

use super::context::{Context, Status};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
pub struct RcContext(pub(crate) NonNull<Context>);

impl RcContext {
    pub fn new<T, F>(size: usize, fun: F) -> Self
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        let cx = Context::new::<T, F>(size, fun);
        RcContext(cx).setup_registers()
    }
    /// # Safety
    /// The access must be unique. There cannot be any
    /// other mutable aliases to this context.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn context(&self) -> &mut Context {
        &mut *self.0.as_ptr()
    }

    pub fn for_os_thread() -> RcContext {
        RcContext(Context::for_os_thread())
    }

    pub fn setup_registers(self) -> Self {
        // println!("{:x}, {:p}", self.stack.bottom(), self.stack.data);

        // self.stack.bottom()
        let cx = unsafe { self.context() };
        cx.registers.sp = cx.stack.bottom();
        cx.registers.arg = cx as *mut _ as u64;
        cx.registers.fun = Self::call_function as u64;
        self
    }

    pub extern "C" fn call_function(link: RcContext, current: RcContext) {
        {
            let cx = unsafe { current.context() };
            assert_eq!(cx.status, Status::New);
            let f = unsafe { cx.fun.as_mut().unwrap() };
            cx.status = Status::Running;
            f(cx.out.cast());
            cx.status = Status::Finished;
            drop(current);
        }
        link.switch_no_save();
    }
    pub fn switch_no_save(self) {
        unsafe { sys::switch_no_save(self) }
    }

    pub fn switch(self, link: RcContext) {
        unsafe { sys::switch_context(link.0, self) }
    }
}

impl Clone for RcContext {
    fn clone(&self) -> Self {
        // SAFETY: The reference doesn't escape the scope
        let cx = unsafe { self.context() };
        cx.refcount += 1;
        RcContext(self.0)
    }
}

impl Drop for RcContext {
    #[track_caller]
    fn drop(&mut self) {
        // SAFETY: The reference doesn't escape the scope
        let cx = unsafe { self.context() };
        cx.refcount -= 1;
        dbg!(cx.refcount);

        if cx.refcount != 0 {
            return;
        }

        // SAFETY: The reference doesn't escape the scope:
        unsafe {
            match cx.status {
                Status::Running => (),
                Status::Taken => (),
                Status::New => cx.fun.drop_in_place(),
                Status::Finished => cx.out.drop_in_place(),
            }
            (cx as *mut Context).drop_in_place();
            let layout = cx.layout;
            println!("dealloc");
            dealloc(self.0.as_ptr().cast(), layout);
        };
    }
}
