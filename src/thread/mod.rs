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
//! ## Spawning a thread
//!
//! A new thread can be spawned using the [`thread::spawn`][`spawn`] function:
//!
//! ```
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
//! ```
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
//! ```
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
//! ```
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
//! Like OS threads, green threads cannot forcibly be made to without leaking memory or
//! causing deadlocks. For this reason, the runtime will always wait for all threads to
//! finish before exiting the program. This means that if a green thread is running an
//! infinite loop, the program will never exit.
//!
//! On shutdown, all io will be disabled.
//!
//! Note that there is no guarantee that any of these cancellation options will cause the
//! thread to exit. When writing applications using `pneuma`, one should be careful not to write
//! infinite loops that do not exit on cancellation.
//!
//! [channels]: pneuma::sync::mpsc
//! [`join`]: JoinHandle::join
//! [`Result`]: pneuma::result::Result
//! [`Ok`]: pneuma::result::Result::Ok
//! [`Err`]: pneuma::result::Result::Err
//! [`thread::current`]: current
//! [`thread::Result`]: Result
//! [`unpark`]: Thread::unpark
//! [`thread::park_timeout`]: park_timeout
//! [`Cell`]: pneuma::cell::Cell
//! [`RefCell`]: pneuma::cell::RefCell
//! [`with`]: LocalKey::with
//! [`thread_local!`]: pneuma::thread_local

pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use repr_context::ReprContext;
use std::{
    cell::Cell,
    fmt::Debug,
    mem::{forget, transmute},
    sync::{atomic::Ordering, Arc},
};

pub(crate) use stack::Stack;

pub mod repr_context;
pub use globals::current;

use pneuma::{
    runtime::{self, SharedQueue},
    sys,
};

pub use self::builder::Builder;
use self::repr_context::{Lifecycle, Status};
pub(crate) mod builder;
pub(crate) mod context;
pub(crate) mod globals;
pub(crate) mod join_handle;
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
/// may result in this thread not being scheduled for an indefinite amount of time. Spurious wakeups
/// cannot be relied on.
///
/// See also [`pneuma::thread::yield_now()`] for a function that yields once cooperatively and reschedules the
/// thread immediately.
pub fn park() {
    runtime::current().park();
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
    park();
}

#[repr(transparent)]
pub struct Thread(pub(crate) Context);

impl Debug for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thread")
            .field("id", &self.id())
            .field("is_cancelled", &self.is_cancelled())
            .finish()
    }
}

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
/// let other_thread_id = other_thread.join();
/// dbg!(other_thread_id);
/// dbg!(thread::current().id());
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
    /// ```ignore
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
    /// parked_thread.join().unwrap();
    /// ```
    /// [`unpark`]: Thread::unpark
    /// [`Waker::wake`]: std::task::Waker::wake
    #[track_caller]
    pub fn unpark(&self) {
        let thread = &self.0;
        if thread.status.get() == Status::Queued {
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
    /// ```ignore
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
    /// ```ignore
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

    pub(crate) fn for_os_thread(sq: Arc<SharedQueue>) -> Thread {
        Thread(Context::for_os_thread(sq))
    }

    pub(crate) fn eq(&self, other: &Thread) -> bool {
        std::ptr::eq(self.0 .0.as_ptr(), other.0 .0.as_ptr())
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub(crate) fn io_result(&self) -> &Cell<Option<i32>> {
        &self.0.io_result
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.0.is_cancelled.get()
    }

    pub(crate) fn into_context(self) -> Context {
        unsafe { transmute(self) }
    }

    pub(crate) fn from_borrowed(cx: Context) -> Thread {
        let out = Thread(cx);
        forget(out.clone());
        out
    }

    fn exit(self) -> ! {
        let rt = pneuma::runtime::current();

        loop {
            rt.executor.remove(&self);
            let Some(thread) = rt.executor.pop() else {
                rt.poll_reactor().ok();
                continue;
            };

            if self.0 .0.as_ptr() != thread.0 .0.as_ptr() {
                let cx = thread.0 .0;
                drop(self);
                drop(thread);
                drop(rt);
                unsafe { sys::load_context(cx) };
            }
        }
    }
}

/// Returns whether the currently running
/// task is cancelled and should exit cooperatively.
pub fn is_cancelled() -> bool {
    dbg!(current()).0.is_cancelled.get()
}

impl Clone for Thread {
    fn clone(&self) -> Self {
        let cx = self.0;
        let count = cx.refcount.get().overflowing_add(1).0;
        cx.refcount.set(count);

        Thread(cx)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let cx = self.0;
        let count = cx.refcount.get() - 1;
        cx.refcount.set(count);
        let layout = cx.layout;

        if count != 0 {
            return;
        }

        match cx.lifecycle.get() {
            Lifecycle::OsThread => {}
            Lifecycle::Running => {
                unreachable!()
            }
            Lifecycle::Taken => (),
            Lifecycle::New => unsafe { cx.fun.drop_in_place() },
            Lifecycle::Finished => unsafe { cx.out.drop_in_place() },
        }

        let count = cx.atomic_refcount.fetch_sub(1, Ordering::Release);

        if count == 1 {
            unsafe {
                self.0 .0.as_ptr().drop_in_place();
                std::alloc::dealloc(self.0 .0.as_ptr().cast(), layout);
            };
        }
    }
}
