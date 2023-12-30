use super::Context;
use super::RcContext;
use std::cell::Cell;
use std::ptr::null_mut;

use std::{mem::forget, ptr::NonNull};

thread_local! {
    /// A fast thread local for quickly accessing the current green thread context.
    static GREEN_THREAD: Cell<*mut Context> = const { Cell::new(null_mut()) };
    /// A slower thread local for accessing the os thread context. It cannot be stored
    static KERNEL_THREAD: RcContext = RcContext::for_os_thread();

}

/// Gets a handle to the green thread that invokes it.
///
/// # Examples
///
/// Getting a handle to the current thread with `thread::current()`:
///
/// ```
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
fn current() -> RcContext {
    if let Some(thread) = _try_green_thread() {
        return thread;
    }
    KERNEL_THREAD.with(|cx| cx.clone())
}

fn _try_green_thread() -> Option<RcContext> {
    let ptr = GREEN_THREAD.get();
    let cx = RcContext(NonNull::new(ptr)?);
    // account for the THREAD reference
    forget(cx.clone());
    Some(cx)
}
