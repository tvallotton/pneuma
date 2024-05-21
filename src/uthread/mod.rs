pub(crate) use context::Context;

use self::lifecycle::OS_THREAD;
pub(crate) use self::repr_context::ReprContext;
use self::thread_id::UThreadId;
pub use join_handle::JoinHandle;
use std::fmt;
use std::sync::atomic::Ordering::*;

pub use builder::Builder;
pub use yield_now::yield_now;

mod builder;
mod context;
mod join_handle;
mod lifecycle;
mod registers;
mod repr_context;
mod thread_id;
mod yield_now;

#[derive(Clone)]
pub struct UThread {
    pub(crate) cx: Context,
}

impl UThread {
    /// Gets the thread's unique identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let other_thread = uthread::spawn(|| {
    ///     uthread::current().id()
    /// });
    ///
    /// let other_thread_id = other_thread.join();
    /// assert!(uthread::current().id() != other_thread_id);
    /// ```
    #[must_use]
    pub fn id(&self) -> UThreadId {
        self.cx.id
    }

    /// Gets the uthread's name.
    ///
    /// # Examples
    ///
    /// Threads by default have no name specified:
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new();
    ///
    /// let handler = builder.spawn(|| {
    ///     assert!(uthread::current().name().is_none());
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    ///
    /// UThread with a specified name:
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new()
    ///     .name("foo".into());
    ///
    /// let handler = builder.spawn(|| {
    ///     assert_eq!(uthread::current().name(), Some("foo"))
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.cx.name.as_deref()
    }

    /// Wakes up the thread to run in the future.
    ///
    /// Every uthread is equipped with some basic low-level event system support, via
    /// the [`park`] function and the [`unpark()`] method. The [`park`] method is
    /// used to cooperatively yield to the scheduler, while the [`unpark`] method
    /// reschedules the thread for execution.
    ///
    /// This is the uthread analog of calling [`Waker::wake`] in async programing,
    /// or `std::thread::Thread::unpark`.
    ///
    /// See the [park documentation][park] for more details.
    ///
    /// # Examples
    ///
    /// ```
    /// use pneuma::uthread;
    ///
    /// let parked_thread = uthread::Builder::new()
    ///     .spawn(|| {
    ///         println!("Parking thread");
    ///         uthread::park();
    ///         println!("Thread unparked");
    ///     })
    ///     .unwrap();
    ///
    /// // Yield so the new thread is spawned
    /// uthread::yield_now();
    ///
    /// println!("Unpark the thread");
    /// parked_thread.thread().unpark();
    ///
    /// parked_thread.join();
    /// ```
    /// [`unpark`]: Thread::unpark
    /// [`Waker::wake`]: std::task::Waker::wake
    pub fn unpark(&self) {
        let already_queued = self
            .cx
            .is_queued
            .compare_exchange(false, true, Release, Relaxed)
            .is_err();

        if !already_queued {
            self.queue();
        }
    }

    #[inline]
    fn queue(&self) {
        if let OS_THREAD = self.cx.lifecycle.load(Relaxed) {
            return;
        }
        pneuma::runtime::current().executor.push(self.clone())
    }

    pub(crate) fn for_os_thread() -> UThread {
        let cx = Context::for_os_thread();
        UThread { cx }
    }
}

/// Gets a handle to the uthread that invokes it.
///
/// # Examples
///
/// Getting a handle to the current uthread with `uthread::current()`:
///
/// ```
/// use pneuma::uthread;
///
/// let handler = uthread::Builder::new()
///     .name("named thread".into())
///     .spawn(|| {
///         let handle = uthread::current();
///         assert_eq!(handle.name(), Some("named thread"));
///     })
///     .unwrap();
///
/// handler.join();
/// ```
#[must_use]
pub fn current() -> UThread {
    pneuma::runtime::current()
        .executor
        .current_thread()
        .lock()
        .unwrap()
        .clone()
}

pub fn park() -> std::io::Result<()> {
    // NOTE: we might never return
    // better not leave any variables undropped
    pneuma::runtime::current().park()
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).unwrap()
}

impl std::fmt::Debug for UThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UThread")
            .field("id", &self.id())
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

impl PartialEq for UThread {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.cx.ptr(), other.cx.ptr())
    }
}
