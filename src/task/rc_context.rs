use std::{
    cell::{Cell, UnsafeCell},
    ptr::NonNull,
};

use crate::{
    debug_registers, sys,
    task::globals::{pop_context, push_context},
};

use super::{
    context::{Context, Status},
    globals::{set_current, set_link},
};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
pub struct RcContext(pub NonNull<Context>);

impl RcContext {
    pub fn new<T, F>(size: usize, fun: F) -> Self
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        let cx = Context::new::<T, F>(size, fun);

        let mut cx = RcContext(cx);
        cx.setup_registers();
        cx
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
    pub fn setup_registers(&mut self) {
        // println!("{:x}, {:p}", self.stack.bottom(), self.stack.data);

        // self.stack.bottom()
        let cx = unsafe { self.context() };
        cx.registers.sp = cx.stack.bottom();
        cx.registers.arg = cx as *mut _ as u64;
        cx.registers.fun = Self::call_function as u64;
    }

    pub extern "C" fn call_function(link: RcContext, current: RcContext) {
        {
            let current = unsafe { current.context() };
            assert_eq!(current.status, Status::New);
            let f = unsafe { current.fun.as_mut().unwrap() };
            current.status = Status::Running;
            f(current.out.cast());
        }
        // link.switch();
    }

    pub fn switch(&self) {
        let (old_link, link) = push_context(self.clone());
        debug_registers();

        unsafe { sys::switch_context(link.context(), self.context()) }
        debug_registers();
        println!("asd");
        unsafe {
            println!("{}", link.context().registers.link);
        }

        pop_context(old_link);
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
    fn drop(&mut self) {
        // SAFETY: The reference doesn't escape the scope
        let cx = unsafe { self.context() };
        cx.refcount += 1;

        if cx.refcount != 0 {
            return;
        }

        // SAFETY: The reference doesn't escape the scope:
        unsafe {
            match cx.status {
                Status::Running => todo!(),
                Status::Taken => (),
                Status::New => cx.fun.drop_in_place(),
                Status::Finished => cx.out.drop_in_place(),
            }
            (cx as *mut Context).drop_in_place();
            let layout = cx.layout;
            dealloc(self.0.as_ptr().cast(), layout);
        };
    }
}
