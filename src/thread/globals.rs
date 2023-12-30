use super::Context;
use super::RcContext;
use super::Thread;
use std::cell::Cell;
use std::ptr::null_mut;

use std::{mem::forget, ptr::NonNull};

thread_local! {
    static THREAD: Cell<Option<RcContext>> = Cell::new(Some(RcContext::for_os_thread()));
}

/// Gets a handle to the thread that invokes it. The thread may be either a
/// green thread or an os thread.
///
/// # Examples
///
/// Getting a handle to the current thread with `thread::current()`:
///
/// ```ignore
/// use pneuma::thread;
///
/// let handler = thread::Builder::new()
///     .name("named thread".into())
///     .spawn(|| {
///         let handle = thread::current();
///         assert_eq!(handle.name(), Some("named thread"));
///     })
///     .unwrap();
///
/// handler.join().unwrap();
/// ```
pub fn current() -> Thread {
    THREAD.with(|cell| {
        let cx = unsafe { cell.take().unwrap_unchecked() };
        let out = cx.clone();
        cell.set(Some(cx));
        Thread(out)
    })
}

pub(crate) fn replace(cx: RcContext) {
    THREAD.with(|cell| {
        cell.set(Some(cx));
    })
}
