use std::{
    arch::asm,
    backtrace::{self, Backtrace},
    cell::{Cell, UnsafeCell},
    ops::Deref,
    ptr::NonNull,
};

use crate::sys;

use super::context::{Context, Status};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
pub(crate) struct RcContext(pub(crate) NonNull<Context>);

impl RcContext {
    pub fn new<T, F>(size: usize, fun: F) -> Self
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        Context::new::<T, F>(size, fun)
    }
    /// # Safety
    /// The access must be unique. There cannot be any
    /// other mutable aliases to this context.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn context(&self) -> &mut Context {
        &mut *self.0.as_ptr()
    }

    pub fn for_os_thread() -> RcContext {
        Context::for_os_thread()
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
            assert_eq!(current.status.get(), Status::New);
            let f = unsafe { cx.fun.as_mut().unwrap() };
            current.status.set(Status::Running);
            f(cx.out.cast());
            current.status.set(Status::Finished);
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
        let count = self.refcount.get() + 1;
        self.refcount.set(count);
        RcContext(self.0)
    }
}

impl Drop for RcContext {
    #[track_caller]
    fn drop(&mut self) {
        let count = self.refcount.get() - 1;
        self.refcount.set(count);

        if count != 0 {
            return;
        }

        // SAFETY: The reference doesn't escape the scope:
        unsafe {
            match self.status.get() {
                Status::Running => (),
                Status::Taken => (),
                Status::New => self.fun.drop_in_place(),
                Status::Finished => self.out.drop_in_place(),
            }
            let layout = self.layout;
            self.0.as_ptr().drop_in_place();
            println!("dealloc");
            dealloc(self.0.as_ptr().cast(), layout);
        };
    }
}
impl Deref for RcContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
