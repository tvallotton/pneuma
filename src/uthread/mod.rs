pub(crate) use context::Context;

pub(crate) use self::repr_context::ReprContext;
use self::{builder::Builder, thread_id::UThreadId};
pub use join_handle::JoinHandle;
use std::sync::atomic::Ordering::*;
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
    /// ```ignore
    /// use pneuma::uthread;
    ///
    /// let other_thread = uthread::spawn(|| {
    ///     thread::current().id()
    /// });
    ///
    /// let other_thread_id = other_thread.join();
    /// assert!(thread::current().id() != other_thread_id);
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
    /// ```ignore
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new();
    ///
    /// let handler = builder.spawn(|| {
    ///     assert!(thread::current().name().is_none());
    /// }).unwrap();
    ///
    /// handler.join();
    /// ```
    ///
    /// UThread with a specified name:
    ///
    /// ```ignore
    /// use pneuma::uthread;
    ///
    /// let builder = uthread::Builder::new()
    ///     .name("foo".into());
    ///
    /// let handler = builder.spawn(|| {
    ///     assert_eq!(thread::current().name(), Some("foo"))
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
    /// Every thread is equipped with some basic low-level event system support, via
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
    pub fn unpark(&self) {
        let Ok(_) = self
            .cx
            .is_queued
            .compare_exchange(false, true, Release, Relaxed)
        else {
            return;
        };
        pneuma::runtime::current().executor.push(self.clone());
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
/// ```ignore
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
        .current()
        .lock()
        .unwrap()
        .clone()
}

pub fn park() -> std::io::Result<()> {
    // NOTE: we might never return
    // better not leave undropped any variables
    pneuma::runtime::current().park()
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).unwrap()
}
