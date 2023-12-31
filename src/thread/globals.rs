use super::Context;
use super::RcContext;
use super::Thread;
use pneuma::runtime;
use std::cell::Cell;
use std::ptr::null_mut;
use std::{mem::forget, ptr::NonNull};

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
    runtime::current().executor.current()
}
