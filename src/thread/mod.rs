//! Green threads.
//!
//! ## The threading model
//!
//! An executing Rust program consists of a collection of native OS threads,
//! each with their own stack and local state. On top of those OS threads
//! a number of green threads can run and be scheduled by the pneuma scheduler.
//! Like OS threads, green threads can be named, and each store their state
//! in their own stack. Because they run on top of OS threads, they share
//! thread local storage.
//!
//! Unlike [`may`](https://docs.rs/may/latest/may/), `pneuma` favors a share
//! nothing architecture over a work stealing one. This means that using thread
//! local storage is permitted and encouraged when using `pneuma`.
//!
//! ## Spawning a thread
//!
//! A new thread can be spawned using the [`thread::spawn`][`spawn`] function:
//!
//! ```no_run
//! use pneuma::thread;
//!
//! thread::spawn(move || {
//!     // some work here
//! });
//! ```
//!
//! In this example, the spawned thread is "detached," which means that there is
//! no way for the program to learn when the spawned thread completes or otherwise
//! terminates.
//!
//! To learn when a thread completes, it is necessary to capture the [`JoinHandle`]
//! object that is returned by the call to [`spawn`], which provides
//! a `join` method that allows the caller to wait for the completion of the
//! spawned thread:
//!
//! ```rust
//! use pneuma::thread;
//!
//! let thread_join_handle = thread::spawn(move || {
//!     "output"
//! });
//! // some work here
//! assert_eq!(thread_join_handle.join(), "output");
//! ```
//!
//! Note that unlike `std`, the [`join`] method returns the output of the thread
//! directly, and resumes unwinding if the child thread panicked.
//!
//! To get the behavior of `std` threads, the `try_join`(JoinHandle::try_join) method
//! can be used:
//! ```no_run
//!  use pneuma::thread;
//!
//! let thread_join_handle = thread::spawn(move || {
//!     panic!()
//! });
//!
//! assert!(thread_join_handle.try_join().is_err());
//! ```
//! This function will return a [`Result`] containing [`Ok`] of the final
//! value produced by the spawned thread, or [`Err`] of the value given to
//! a call to [`panic!`] if the thread panicked.
//!
//!
//! ## Configuring threads
//!
//! A new thread can be configured before it is spawned via the [`Builder`] type,
//! which currently allows you to set the name and stack size for the thread:
//!
//! ```no_run
//! use pneuma::thread;
//!
//! thread::Builder::new().name("thread1".to_string()).spawn(move || {
//!     println!("Hello, world!");
//! }).unwrap();
//! ```
//!
//! ## The `Thread` type
//!
//! Threads are represented via the [`Thread`] type, which you can get in one of
//! two ways:
//!
//! * By spawning a new thread, e.g., using the [`thread::spawn`][`spawn`]
//!   function, and calling [`thread`][`JoinHandle::thread`] on the [`JoinHandle`].
//! * By requesting the current thread, using the [`thread::current`] function.
//!
//! The [`thread::current`] function is available even for threads not spawned
//! by the APIs of this module.
//!
//!
//! ## Naming threads
//!
//! Threads are able to have associated names for identification purposes. By default, spawned
//! threads are unnamed. To specify a name for a thread, build the thread with [`Builder`] and pass
//! the desired thread name to [`Builder::name`]. To retrieve the thread name from within the
//! thread, use [`Thread::name`]. A couple of examples where the name of a thread gets used:
//!
//! * If a panic occurs in a named thread, the thread name will be printed in the panic message.
//!
//!
//! ## Stack size
//!
//! The default stack size is platform-dependent and subject to change.
//! Currently, it is 16 KiB on all platforms.
//!
//! In order to set the stack size use the thread with [`Builder`] and pass
//! the desired stack size to [`Builder::stack_size`].
//!
//!
//! ## Cancellation
//!
//! Like OS threads, green threads cannot be easily cancelled without leaking memory or
//! causing deadlocks. For this reason, the runtime will always wait for all threads to
//! finish before exiting the program. This means that if a green thread is running an
//! infinite loop, the program will never exit.
//!
//! However, the runtime offers a mechanism for tasks to exit cooperatively. This is achieved
//! through the `Thread::cancel` method. The `cancel` method supports two different mechanisms
//! for cancellation:
//!
//! 1. [`Cancel::FlagOnly`]: Allows the tasks to check for cancellation, through the [`is_cancelled`] method.
//! 2. [`Cancel::DisableIo`]: Will cause all pending async io to yield immediately with an error.
//!
//! ```no_run
//! use pneuma::thread;
//!
//! let thread = thread::spawn(|| {
//!     while !thread::is_cancelled() {
//!         yield_now().await;
//!     }
//! });
//! thread::yield_now();
//! thread.cancel(thread::Cancel::FlagOnly);
//! ```
//! Note that there is no guarantee that any of these cancellation options will cause the
//! thread to exit. When writing applications using `pneuma`, one should be careful not to write
//! infinite loops that do not exit on cancellation.
//!
//! [channels]: crate::sync::mpsc
//! [`join`]: JoinHandle::join
//! [`Result`]: crate::result::Result
//! [`Ok`]: crate::result::Result::Ok
//! [`Err`]: crate::result::Result::Err
//! [`thread::current`]: current
//! [`thread::Result`]: Result
//! [`unpark`]: Thread::unpark
//! [`thread::park_timeout`]: park_timeout
//! [`Cell`]: crate::cell::Cell
//! [`RefCell`]: crate::cell::RefCell
//! [`with`]: LocalKey::with
//! [`thread_local!`]: crate::thread_local

pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use rc_context::RcContext;
use std::{any::Any, cell::Cell};

pub(crate) use stack::Stack;

pub mod context;
pub use globals::current;

use crate::runtime;

pub use self::builder::Builder;
use self::context::{Lifecycle, Status};
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
/// Some good use cases for [`yield_now`] are:
/// 1. Calling it periodically while running a
/// long CPU bound computation, so the scheduler can respond to events.
/// 2. Yield after a [`Mutex`] lock is released, to make sure another thread gets to acquire
/// it next.
/// 3. Yielding after unparking or spawning a thread, but you still have more work to do.
///
/// Some bad use cases are:
/// 1. Calling it on a loop until a condition is met.
/// 2. Implementing a spinlock.
/// 3. Calling it while you don't have any more work to do.
///
///
/// # Examples
///
/// ```no_run
/// use pneuma::thread;
///
/// thread::yield_now();
/// ```
/// [`join`]: Thread::join
/// [`Mutex`]: std::sync::Mutex
/// [`channel`]: std::sync::mpsc::channel
pub fn yield_now() {
    let current = current();
    current.unpark();
    park();
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
/// ```no_run
/// use pneuma::thread;
///
/// let other_thread = thread::spawn(|| {
///     thread::current().id()
/// });
///
/// let other_thread_id = other_thread.join();
/// assert!(thread::current().id() != other_thread_id);
/// ```
///
/// [`id`]: Thread::id
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub struct ThreadId(usize);

impl Thread {
    /// Wakes up the thread to run in the future.
    ///
    /// Every thread is equipped with some basic low-level event system support, via
    /// the [`park`] function and the [`unpark()`] method. The [`park`] method is
    /// used to cooperatively yield to the scheduler, while the [`unpark`] method
    /// reschedules the thread for execution.
    ///
    /// This is the green thread analog of calling [`Waker::wake`].
    ///
    /// See the [park documentation][park] for more details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pneuma::thread;
    ///
    /// let parked_thread = thread::Builder::new()
    ///     .spawn(|| {
    ///         println!("Parking thread");
    ///         thread::park();
    ///         println!("Thread unparked");
    ///     })
    ///     .unwrap();
    ///
    /// // Yield so the new thread is spawned
    /// thread::yield_now();
    ///
    /// println!("Unpark the thread");
    /// parked_thread.thread().unpark();
    ///
    /// parked_thread.join();
    /// ```
    /// [`unpark`]: Thread::unpark
    /// [`Waker::wake`]: std::task::Waker::wake
    pub fn unpark(&self) {
        let thread = &self.0;
        if thread.status.get() == Status::Queued {
            return;
        }
        if let Lifecycle::Finished | Lifecycle::Taken = thread.lifecycle.get() {
            return;
        }

        thread.status.set(Status::Queued);
        runtime::current().executor.push(self.clone());
    }
    /// Gets the thread's name.
    ///
    /// For more information about named threads, see
    /// [this module-level documentation][naming-threads].
    ///
    /// # Examples
    ///
    /// Threads by default have no name specified:
    ///
    /// ```no_run
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new();
    ///
    /// let handler = builder.spawn(|| {
    ///     assert!(thread::current().name().is_none());
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    ///
    /// Thread with a specified name:
    ///
    /// ```no_run
    /// use pneuma::thread;
    ///
    /// let builder = thread::Builder::new()
    ///     .name("foo".into());
    ///
    /// let handler = builder.spawn(|| {
    ///     assert_eq!(thread::current().name(), Some("foo"))
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    ///
    /// [naming-threads]: ./index.html#naming-threads
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
