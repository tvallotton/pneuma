use std::{
    any::Any,
    io,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::NonNull,
};

use pneuma::sys;

use crate::thread::{park, Thread};

use super::{
    builder::Builder,
    context::{Context, Lifecycle},
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
        registers[0] = self.stack.bottom();
        registers[1] = sys::start_coroutine as u64;
        registers[12] = Self::start_coroutine as u64;
        self
    }

    pub extern "C" fn start_coroutine(link: NonNull<Context>, current: NonNull<Context>) {
        {
            let current = Self::from_borrowed(current);
            let f = unsafe { current.fun.as_mut().unwrap() };
            current.lifecycle.set(Lifecycle::Running);
            f(current.out.cast());
            current.lifecycle.set(Lifecycle::Finished);
            current.join_waker.take().as_ref().map(Thread::unpark);
        }

        loop {
            park();
        }
    }

    fn from_borrowed(cx: NonNull<Context>) -> Self {
        let cx = Self(cx);
        std::mem::forget(cx.clone());
        cx
    }
}

impl Clone for RcContext {
    fn clone(&self) -> Self {
        let count = self.refcount.get() + 1;
        self.refcount.set(count);

        RcContext(self.0)
    }
}

impl Drop for RcContext {
    // #[track_caller]
    fn drop(&mut self) {
        let count = self.refcount.get() - 1;
        self.refcount.set(count);
        let layout = self.layout;
        if count != 0 {
            return;
        }

        match self.lifecycle.get() {
            Lifecycle::OsThread => {}
            Lifecycle::Running => {
                dbg!("running");
                Thread(self.clone()).unpark();
                park();
            }
            Lifecycle::Taken => (),
            Lifecycle::New => unsafe { self.fun.drop_in_place() },
            Lifecycle::Finished => unsafe { self.out.drop_in_place() },
        }

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
