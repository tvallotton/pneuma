use std::{
    any::Any,
    io,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::NonNull,
    sync::Arc,
};

use pneuma::sys;

use crate::{runtime::SharedQueue, thread::Thread};

use super::{
    builder::Builder,
    repr_context::{Lifecycle, ReprContext},
};

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Context(pub(crate) NonNull<ReprContext>);

impl Context {
    pub fn new<T, F>(f: F, sq: Arc<SharedQueue>, builder: Builder) -> io::Result<Context>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        ReprContext::new::<T, _>(type_errased(f), sq, builder)
    }

    pub(crate) fn for_os_thread(sq: Arc<SharedQueue>) -> Context {
        ReprContext::for_os_thread(sq)
    }

    pub fn setup_registers(self) -> Self {
        let registers = unsafe { &mut *self.registers.get() };
        registers[0] = self.stack.bottom();
        registers[1] = sys::start_coroutine as u64;
        registers[2] = sys::start_coroutine as u64;
        registers[11] = Self::thread_start as u64;
        self
    }

    pub extern "C" fn thread_start(cx: Context) {
        let current = Thread::from_borrowed(cx);
        {
            let current = current.0;
            assert_eq!(current.lifecycle.get(), Lifecycle::New);
            let f = unsafe { current.fun.as_mut().unwrap() };
            current.lifecycle.set(Lifecycle::Running);
            f(current.out.cast());
            current.lifecycle.set(Lifecycle::Finished);
            current.join_waker.take().as_ref().map(Thread::unpark);
        }

        current.exit();
    }

    pub fn as_thread(&self) -> &Thread {
        unsafe { std::mem::transmute(self) }
    }
}

fn type_errased<F, T>(f: F) -> impl FnMut(*mut ())
where
    F: FnOnce() -> T,
{
    let mut f = Some(f);
    move |out: *mut ()| {
        let closure = f.take().unwrap();
        let res = catch_unwind(AssertUnwindSafe(closure));
        unsafe {
            out.cast::<Result<T, Box<dyn Any + Send + 'static>>>()
                .write(res)
        }
    }
}

impl AsRef<ReprContext> for Context {
    fn as_ref(&self) -> &ReprContext {
        unsafe { &*self.0.as_ptr() }
    }
}

impl Deref for Context {
    type Target = ReprContext;
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}
