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

use crate::runtime;

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
/// should be prepared for this possibility. If the thread was unparked while it is still running,
/// this will cause the thread to be rescheduled. This means that unpark followed by park could result
/// in the second call returning immediately.
///
/// Note that is the caller's responsibility to schedule a call to [`Thread::unpark`]. Not doing so
/// may result in this thread not being scheduled for an indefinite amount of time. Spurious wake ups
/// cannot be relied on.
///
/// See also [`pneuma::thread::yield_now()`] for a function that yields once cooperatively and reschedules the
/// thread immediately.
pub fn park() {
    runtime::current().park()
}

/// Cooperatively gives up a timeslice to the pneuma scheduler.
/// This function is the green thread analog to [`std::thread::yield_now()`].
///
/// This will yield so another green thread gets to run, signaling
/// that the calling thread is willing to give up its remaining timeslice
/// so that pneuma may schedule other threads on the CPU.
///
/// A drawback of yielding in a loop is that if the scheduler does not have any
/// other ready threads to run, the thread will effectively
/// busy-wait, which wastes CPU time and energy.
///
/// Therefore, when waiting for events of interest, a programmer's first
/// choice should be to use synchronization devices such as [`channel`]s,
/// [`Mutex`]es or [`join`] since these primitives are event based,
/// giving up the CPU until the event of interest has occurred, and signaling
/// to the scheduler which thread should run next.
///
/// Some good use cases for [`yield_now``] are:
/// 1. Calling it periodically while running a
/// long CPU bound computation, so the scheduler can respond to events.
/// 2. Yield after a [`Mutex`] lock is released, to make sure another thread gets to acquire
/// it next.
/// Some bad use cases are:
/// 1. Calling it on a loop until a condition is met.
/// 2. Implementing a spinlock
///
/// # Examples
///
/// ```
/// use pneuma::thread;
///
/// thread::yield_now();
/// ```
/// [`join`]: Thread::join
/// [`Mutex`]: std::sync::Mutex
/// [`channel`]: std::sync::mpsc::channel
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
        runtime::current().executor.push(self.clone());
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    pub(crate) fn status(&self) -> &Cell<Status> {
        &self.0.status
    }

    pub fn id(&self) -> ThreadId {
        ThreadId(self.0 .0.as_ptr() as usize)
    }

    pub(crate) fn for_os_thread() -> Thread {
        Thread(RcContext::for_os_thread())
    }
}
