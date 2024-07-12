use std::{
    any::Any,
    io,
    mem::transmute,
    ops::Deref,
    panic::{catch_unwind, AssertUnwindSafe},
    process::abort,
    ptr::NonNull,
    sync::atomic::Ordering::*,
    sync::atomic::{self},
};

use pneuma::{runtime::current, sys};

use super::{
    builder::Builder,
    lifecycle::{FINISHED, NEW, OS_THREAD, RUNNING, TAKEN},
    repr_context::ReprContext,
    UThread,
};
#[repr(transparent)]
pub(crate) struct Context {
    pub ptr: NonNull<ReprContext>,
}

impl Context {
    pub fn new<T, F>(f: F, builder: Builder) -> io::Result<Context>
    where
        F: FnOnce() -> T + 'static,
        T: Send + 'static,
    {
        unsafe { Self::new_unchecked(f, builder) }
    }

    pub unsafe fn new_unchecked<T, F>(f: F, builder: Builder) -> io::Result<Context>
    where
        F: FnOnce() -> T,
    {
        Ok(ReprContext::new::<T, _>(type_errased(f), builder)?.setup_registers())
    }

    pub fn setup_registers(self) -> Self {
        let registers = unsafe { &mut *self.registers.get() };
        *registers = [Self::uthread_start as u64; 19];
        registers[0] = self.stack.bottom();
        registers[11] = Self::uthread_start as u64;
        self
    }

    pub fn ptr(&self) -> *mut ReprContext {
        self.ptr.as_ptr()
    }

    pub fn for_os_thread() -> Context {
        ReprContext::for_os_thread()
    }

    pub fn switch_to(self, new: Context) -> Result<(), ()> {
        new.lock()?;
        let [old, _] = unsafe { sys::switch_context(self, new) };
        old.unlock();

        Ok(())
    }

    pub extern "C" fn uthread_start(old: Context, new: Context) {
        new.is_queued.store(false, Relaxed);
        old.unlock();

        new.run_uthread();

        loop {
            pneuma::uthread::park().ok();
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

    pub fn wake_joiner(&self) {
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

    pub fn unlock(self) {
        self.is_running.store(false, Release);

        if self.has_exited() {
            self.wake_joiner();
            current().clean_stacks();
        }
    }

    pub fn has_exited(&self) -> bool {
        matches!(self.lifecycle.load(Acquire), FINISHED | TAKEN)
    }

    pub fn as_uthread(&self) -> &UThread {
        unsafe { transmute(self) }
    }
}

fn type_errased<'a, F, T>(f: F) -> impl FnMut(*mut ()) + 'a
where
    F: FnOnce() -> T + 'a,
    T: 'a,
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
            RUNNING => return self.as_uthread().unpark(),
            _ => unreachable!(),
        }

        unsafe {
            self.ptr.as_ptr().drop_in_place();
            std::alloc::dealloc(self.ptr().cast(), layout);
        };
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}
