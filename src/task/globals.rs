use super::Context;
use super::RcContext;
use std::cell::Cell;
use std::ptr::null_mut;
use std::thread::LocalKey;
use std::{mem::forget, ptr::NonNull};

thread_local! {
    static LINK: Cell<*mut Context> = const { Cell::new(null_mut()) };
    static CURRENT: Cell<*mut Context> = const { Cell::new(null_mut()) };
}

pub fn get_link() -> Option<RcContext> {
    read_key(&LINK)
}

pub fn set_link(new: Option<RcContext>) -> Option<RcContext> {
    set_key(&LINK, new)
}
pub fn read_current() -> Option<RcContext> {
    read_key(&CURRENT)
}

pub fn take_current(new: Option<RcContext>) -> Option<RcContext> {
    set_key(&CURRENT, new)
}

fn read_key(key: &'static LocalKey<Cell<*mut Context>>) -> Option<RcContext> {
    let cx = key.get();
    let cx = NonNull::new(cx)?;
    let cx = RcContext(cx);
    let out = cx.clone();
    forget(cx);
    Some(out)
}
fn set_key(
    key: &'static LocalKey<Cell<*mut Context>>,
    new: Option<RcContext>,
) -> Option<RcContext> {
    let cx = new.map(|cx| cx.0.as_ptr()).unwrap_or(null_mut());
    let cx = key.replace(cx);
    let cx = NonNull::new(cx)?;
    let cx = RcContext(cx);
    Some(cx)
}
