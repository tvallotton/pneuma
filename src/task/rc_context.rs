use std::{cell::Cell, ptr::NonNull};

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
        RcContext(Context::new::<T, F>(size, fun))
    }

    pub fn refcount(&self) -> &Cell<u64> {
        unsafe { &self.0.as_ref().refcount }
    }
}

impl Clone for RcContext {
    fn clone(&self) -> Self {
        // SAFETY: The reference doesn't escape the scope
        let count = self.refcount().get();
        self.refcount().set(count + 1);
        RcContext(self.0)
    }
}

impl Drop for RcContext {
    fn drop(&mut self) {
        let refcount = self.refcount().get() - 1;
        self.refcount().set(refcount);
        if refcount != 0 {
            return;
        }
        // SAFETY: The reference doesn't escape the scope:
        unsafe {
            match self.0.as_mut().status {
                Status::Running => todo!(),
                Status::Taken => (),
                Status::New => self.0.as_mut().fun.drop_in_place(),
                Status::Finished => self.0.as_mut().out.drop_in_place(),
            }

            self.0.as_ptr().drop_in_place();
            
            dealloc(self.0.as_ptr().cast(), self.0.as_ref().layout);
        };
    }
}
