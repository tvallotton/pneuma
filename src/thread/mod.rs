pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use rc_context::RcContext;
use std::{
    any::Any,
    cell::Cell,
    io,
    marker::PhantomData,
    panic::{catch_unwind, AssertUnwindSafe},
};

pub(crate) use stack::Stack;

pub mod context;
pub use globals::current;

pub use self::builder::Builder;
use self::context::Status;
pub(crate) mod builder;
pub(crate) mod globals;
pub(crate) mod join_handle;
pub(crate) mod rc_context;
pub(crate) mod registers;
pub(crate) mod stack;

pub fn spawn<T, F>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + 'static,
    T: 'static,
{
    Builder::new().spawn(f).unwrap()
}

/// Wait unless or until the current thread is unparked by someone calling [`Thread::unpark`].
///
/// A call to park does not guarantee that the thread will remain parked forever, and callers
/// should be prepared for this possibility.
///
/// Note that is the caller's responsibility to schedule a call to [`Thread::unpark`]. Not doing so
/// may result in this thread not being scheduled for an indefinite amount of time. Spurious wake ups
/// cannot be relied on.
///
/// See also [`pneuma::thread::yield_now()`] for a function that yields once and reschedules the
/// thread immediately.
pub fn park() {
    let this = current();
    while let Some(next) = this.0.runtime.executor.pop() {
        if this.id() == next.id() {
            continue;
        }

        return next.0.switch(this.0);
    }
    this.0.runtime.poll_reactor()
}

pub fn yield_now() {
    current().unpark();
    park()
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Thread(pub(crate) RcContext);

/// A unique identifier for a running thread.
///
/// A `ThreadId` is an opaque object that uniquely identifies each thread
/// created during the lifetime of a process. Unlinke std's `ThreadId`, Pneuma's
/// `ThreadId`s may be reused after a thread terminates. A `ThreadId`
/// can be retrieved from the [`id`] method on a [`Thread`].
///
/// # Examples
///
/// ```ignore
/// use pneuma::thread;
///
/// let other_thread = thread::spawn(|| {
///     thread::current().id()
/// });
///
/// let other_thread_id = other_thread.join().unwrap();
/// assert!(thread::current().id() != other_thread_id);
/// ```
///
/// [`id`]: Thread::id
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct ThreadId(usize);

impl Thread {
    pub fn unpark(&self) {
        let thread = &self.0;
        if thread.status.get() == Status::Queued {
            return;
        }
        thread.status.set(Status::Queued);
        thread.runtime.executor.push(self.clone());
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    pub fn id(&self) -> ThreadId {
        ThreadId(self.0 .0.as_ptr() as usize)
    }
}
