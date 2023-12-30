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

pub mod join_handle;
pub mod rc_context;
pub mod registers;
pub mod stack;
pub mod globals;

pub fn spawn<T, F>() -> JoinHandle<T>
where
    F: FnOnce() -> T,
{
    todo!()
}

// Tasks underneath are basically reference counted contexts
// note that unlike
#[derive(Clone)]
pub(crate) struct Task<T>(pub RcContext, PhantomData<T>);

impl<T> Task<T> {
    pub fn new<F>(size: usize, f: F) -> io::Result<Task<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let cx = RcContext::new::<T, _>(size, f)?;
        Ok(Task(cx, PhantomData))
    }

    pub fn switch(self, link: RcContext) {
        self.0.switch(link);
    }
}
