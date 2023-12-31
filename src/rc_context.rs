use std::{
    any::Any,
    io,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::NonNull,
};

use pneuma::sys;

use super::{
    builder::Builder,
    context::{Context, Status},
};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
pub(crate) struct RcContext(pub(crate) NonNull<Context>);

impl RcContext {
    pub fn new<T, F>(f: F, builder: Builder) -> io::Result<RcContext>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        // The purpose of this closure is to convert the FnOnce into an FnMut
        // And to type errase the closure.
        let mut f = Some(f);
        let fun = move |out: *mut ()| {
            let closure = f.take().unwrap();
            let res = catch_unwind(AssertUnwindSafe(closure));
            unsafe {
                out.cast::<Result<T, Box<dyn Any + Send + 'static>>>()
                    .write(res)
            }
        };
        Context::new::<T, _>(fun, builder)
    }

    pub(crate) fn for_os_thread() -> RcContext {
        Context::for_os_thread()
    }

    pub fn setup_registers(self) -> Self {
        let registers = unsafe { &mut *self.registers.get() };
        registers.sp = self.stack.bottom();
        registers.arg = self.0.as_ptr() as u64;
        registers.fun = Self::call_function as u64;
        self
    }

    pub extern "C" fn call_function(link: RcContext, current: RcContext) {
        {
            assert_eq!(current.status.get(), Status::New);
            let f = unsafe { current.fun.as_mut().unwrap() };
            current.status.set(Status::Running);
            f(current.out.cast());
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
        let layout = self.layout;

        if count != 0 {
            return;
        }
        dbg!();
        match self.status.get() {
            Status::OsRcContext => (),
            Status::Running => return self.runtime.executor.push(self.clone()),
            Status::Taken => (),
            Status::New => unsafe { self.fun.drop_in_place() },
            Status::Finished => unsafe { self.out.drop_in_place() },
        }
        dbg!();
        unsafe {
            self.0.as_ptr().drop_in_place();
            dealloc(self.0.as_ptr().cast(), layout);
        };
    }
}
impl AsRef<Context> for RcContext {
    fn as_ref(&self) -> &Context {
        unsafe { &*self.0.as_ptr() }
    }
}

impl Deref for RcContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
