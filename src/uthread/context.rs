use std::{
    any::Any,
    io,
    mem::forget,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    process::abort,
    ptr::NonNull,
    sync::atomic::Ordering::*,
    sync::atomic::{self},
};

use crate::sys;

use super::{
    builder::Builder,
    lifecycle::{self, FINISHED, NEW, OS_THREAD, RUNNING, TAKEN},
    repr_context::ReprContext,
    UThread,
};

pub(crate) struct Context {
    pub ptr: NonNull<ReprContext>,
}

impl Context {
    pub fn new<T, F>(f: F, builder: Builder) -> io::Result<Context>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        Ok(ReprContext::new::<T, _>(type_errased(f), builder)?.setup_registers())
    }

    pub fn setup_registers(self) -> Self {
        let registers = unsafe { &mut *self.registers.get() };
        registers[0] = self.stack.bottom();
        registers[1] = sys::start_coroutine as u64;
        registers[2] = sys::start_coroutine as u64;
        registers[11] = Self::uthread_start as u64;
        self
    }

    pub fn ptr(&self) -> *mut ReprContext {
        self.ptr.as_ptr()
    }

    pub fn for_os_thread() -> Context {
        ReprContext::for_os_thread()
    }

    pub extern "C" fn switch_to(self, new: Context) -> Result<(), ()> {
        new.lock()?;

        let [old, _] = unsafe { sys::switch_context(self, new) };

        old.unlock();

        Ok(())
    }

    pub extern "C" fn uthread_start(old: Context, new: Context) {
        old.unlock();
        drop(old);

        new.run_uthread();

        loop {
            pneuma::uthread::park();
        }
    }

    pub fn run_uthread(self) {
        let f = unsafe { self.fun.as_mut().unwrap() };

        self.lifecycle.store(RUNNING, Release);

        f(self.out.cast());
        self.lifecycle.store(FINISHED, Release);
        self.join_waker
            .lock()
            .unwrap()
            .as_ref()
            .map(UThread::unpark);
    }

    pub fn lock(&self) -> Result<bool, ()> {
        self.is_running
            .compare_exchange(false, true, Acquire, Relaxed)
            .map_err(|_| ())
    }

    pub fn unlock(&self) {
        self.is_running.store(false, Release)
    }

    pub fn from_borrowed(ptr: NonNull<ReprContext>) -> Self {
        let cx = Context { ptr };
        forget(cx.clone());
        cx
    }
}

fn type_errased<F, T>(f: F) -> impl FnMut(*mut ()) + 'static
where
    F: FnOnce() -> T + 'static,
    T: 'static,
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

impl Deref for Context {
    type Target = ReprContext;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let old_refcount = self.refcount.fetch_add(1, Relaxed);

        if old_refcount == u64::MAX {
            abort()
        }

        Self { ptr: self.ptr }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if self.refcount.fetch_sub(1, Release) != 1 {
            return;
        }

        atomic::fence(Acquire);

        let layout = self.layout;

        match self.lifecycle.load(Acquire) {
            OS_THREAD => {}
            TAKEN => (),
            NEW => unsafe { self.fun.drop_in_place() },
            FINISHED => unsafe { self.out.drop_in_place() },
            RUNNING | _ => {
                unreachable!()
            }
        }

        unsafe {
            self.ptr.as_ptr().drop_in_place();
            std::alloc::dealloc(self.ptr().cast(), layout);
        };
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}
