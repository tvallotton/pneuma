use super::Context;
use super::RcContext;
use std::cell::Cell;
use std::ptr::null_mut;
use std::thread::LocalKey;
use std::{mem::forget, ptr::NonNull};

thread_local! {

    static THREAD: Cell<*mut Context> = const { Cell::new(null_mut()) };

}

// fn current() {}

fn _try_current() -> Option<RcContext> {
    let ptr = THREAD.get();
    let cx = RcContext(NonNull::new(ptr)?);
    // account for the THREAD reference
    forget(cx.clone());
    Some(cx)
}
