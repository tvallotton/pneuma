use std::{
    any::Any,
    io,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::NonNull,
};

use pneuma::sys;

use crate::thread::Thread;

use super::{
    builder::Builder,
    repr_context::{Lifecycle, ReprContext},
};
use std::alloc::dealloc;

/// This is a reference counted context.
/// An RcContext may be either a type errased Task, or
/// a OS thread context.
#[repr(C)]
pub(crate) struct Context(pub(crate) NonNull<ReprContext>);

impl Context {
    pub fn new<T, F>(f: F, builder: Builder) -> io::Result<Context>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        ReprContext::new::<T, _>(type_errased(f), builder)
    }

    pub(crate) fn for_os_thread() -> Context {
        ReprContext::for_os_thread()
    }

    pub fn setup_registers(self) -> Self {
        let registers = unsafe { &mut *self.registers.get() };
        registers[0] = self.stack.bottom();
        registers[1] = sys::start_coroutine as u64;
        registers[2] = sys::start_coroutine as u64;
        registers[11] = Self::thread_start as u64;
        self
    }

    pub extern "C" fn thread_start(cx: NonNull<ReprContext>) {
        let current = Self::from_borrowed(cx);
        {
            assert_eq!(current.lifecycle.get(), Lifecycle::New);
            let f = unsafe { current.fun.as_mut().unwrap() };
            current.lifecycle.set(Lifecycle::Running);
            f(current.out.cast());
            current.lifecycle.set(Lifecycle::Finished);
            current.join_waker.take().as_ref().map(Thread::unpark);
        }

        current.exit();
    }

    fn exit(self) -> ! {
        let rt = pneuma::runtime::current();

        loop {
            rt.executor.remove(self.as_thread());
            let Some(thread) = rt.executor.pop() else {
                rt.poll_reactor().ok();
                continue;
            };

            if self.0.as_ptr() != thread.0 .0.as_ptr() {
                let cx = thread.0 .0;
                drop(self);
                drop(thread);
                drop(rt);
                unsafe { sys::load_context(cx) };
            }
        }
    }

    pub fn from_borrowed(cx: NonNull<ReprContext>) -> Self {
        let cx = Self(cx);
        std::mem::forget(cx.clone());
        cx
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

impl Clone for Context {
    fn clone(&self) -> Self {
        // SAFETY: The reference doesn't escape the scope
        let count = self.refcount.get() + 1;
        self.refcount.set(count);
        Context(self.0)
    }
}

impl Drop for Context {
    #[track_caller]
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
                unreachable!()
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
