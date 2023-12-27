use std::{
    cell::{Cell, UnsafeCell},
    ptr::NonNull,
};

use super::context::{Context, Status};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
pub struct RcContext(pub NonNull<Context>);

impl RcContext {
    pub fn new<T, F>(size: usize, fun: F) -> Self
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        let cx = Context::new::<T, F>(size, fun);
        RcContext(cx)
    }
    /// # Safety
    /// The access must be unique. There cannot be any
    /// other mutable aliases to this context.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn context(&self) -> &mut Context {
        &mut *self.0.as_ptr()
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
