pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use rc_context::Thread;
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

// Tasks underneath are basically reference counted contexts
// note that unlike
#[derive(Clone)]
pub(crate) struct Task<T>(pub Thread, PhantomData<T>);

impl<T> Task<T> {
    pub fn new<F>(f: F, builder: Builder) -> io::Result<Task<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = Thread::new::<T, _>(f, builder)?;
        Ok(Task(cx, PhantomData))
    }

    pub fn switch(self, link: Thread) {
        self.0.switch(link);
    }
}
