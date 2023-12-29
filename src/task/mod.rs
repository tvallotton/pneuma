pub(crate) use context::Context;
pub use join_handle::JoinHandle;
pub(crate) use rc_context::RcContext;
use std::{
    any::Any,
    arch::asm,
    cell::Cell,
    marker::PhantomData,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::{null_mut, NonNull},
};

use crate::sys;

pub mod context;
// pub mod globals;
pub mod join_handle;
pub mod rc_context;
pub mod registers;
mod stack;

// Tasks underneath are basically reference counted contexts
// note that unlike
#[derive(Clone)]
pub(crate) struct Task<T>(pub RcContext, PhantomData<T>);

impl<T> Task<T> {
    pub fn new<F>(size: usize, f: F) -> Task<T>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        // The purpose of this closure is to convert the FnOnce into an FnMut
        // And to type errase the closure.
        let mut f = Some(f);
        let f = move |out: *mut ()| {
            let closure = f.take().unwrap();
            let res = catch_unwind(AssertUnwindSafe(closure));
            unsafe {
                out.cast::<Result<T, Box<dyn Any + Send + 'static>>>()
                    .write(res)
            }
            dbg!()
        };

        Task(RcContext::new::<T, _>(size, f), PhantomData)
    }

    pub fn switch(self, link: RcContext) {
        self.0.switch(link);
    }
}
