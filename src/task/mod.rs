pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use rc_context::RcContext;
use std::{
    any::Any,
    io,
    marker::PhantomData,
    panic::{catch_unwind, AssertUnwindSafe},
};

pub(crate) use stack::Stack;

pub mod context;
pub use globals::current;

pub use self::builder::Builder;
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

#[derive(Clone)]
pub struct Thread(RcContext);

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
        self.0.runtime.executor.push(self.clone().0);
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    pub fn id(&self) -> ThreadId {
        ThreadId(self.0 .0.as_ptr() as usize)
    }
}

// Tasks underneath are basically reference counted contexts
// note that unlike
#[derive(Clone)]
pub(crate) struct Task<T>(pub RcContext, PhantomData<T>);

impl<T> Task<T> {
    pub fn new<F>(f: F, builder: Builder) -> io::Result<Task<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = RcContext::new::<T, _>(f, builder)?;
        Ok(Task(cx, PhantomData))
    }

    pub fn switch(self, link: RcContext) {
        self.0.switch(link);
    }
}
