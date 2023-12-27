pub use rc_context::RcContext;
use std::{
    cell::Cell,
    marker::PhantomData,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::{null_mut, NonNull},
};

use self::context::Context;

pub mod context;
pub mod globals;
pub mod rc_context;
pub mod registers;
mod stack;

// Tasks underneath are basically reference counted contexts
// note that unlike
#[derive(Clone)]
pub(crate) struct Task<T>(RcContext, PhantomData<T>);

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
            let closure = (&mut f).take().unwrap();
            let res = catch_unwind(AssertUnwindSafe(|| closure));
            unsafe { *out.cast() = res }
        };

        Task(RcContext::new::<T, _>(size, f), PhantomData)
    }

    pub fn start(&self) -> ! {
        todo!()
    }

    pub fn resume(&self) -> ! {
        todo!()
    }
}
