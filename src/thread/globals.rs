use super::Context;
use super::RcContext;
use super::Thread;
use std::cell::Cell;
use std::ptr::null_mut;

use std::{mem::forget, ptr::NonNull};

thread_local! {
    /// A fast thread local for quickly accessing the current green thread context.
    static GREEN_THREAD: Cell<*mut Context> = const { Cell::new(null_mut()) };
    /// A slower thread local for accessing the os thread context. It cannot be stored
    static KERNEL_THREAD: RcContext = RcContext::for_os_thread();

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
    try_green_thread().unwrap_or_else(os_thread)
}

pub(crate) fn os_thread() -> Thread {
    KERNEL_THREAD.with(|cx| Thread(cx.clone()))
}

pub(crate) fn try_green_thread() -> Option<Thread> {
    let ptr = GREEN_THREAD.get();
    let cx = RcContext(NonNull::new(ptr)?);
    // account for the THREAD reference
    forget(cx.clone());
    Some(Thread(cx))
}
