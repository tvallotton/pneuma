use super::Context;
use super::RcContext;
use std::cell::Cell;
use std::ptr::null_mut;
use std::thread::LocalKey;
use std::{mem::forget, ptr::NonNull};

thread_local! {
    static LINK: Cell<*mut Context> = const { Cell::new(null_mut()) };
    static CURRENT: Cell<*mut Context> = const { Cell::new(null_mut()) };
    static THREAD_OS: Cell<*mut Context> = const { Cell::new(null_mut()) };
}

pub fn push_context(new_current: RcContext) -> (Option<RcContext>, RcContext) {
    let new_link = set_current(Some(new_current));
    let new_link = new_link.unwrap_or_else(RcContext::for_os_thread);
    let old_link = set_link(Some(new_link.clone()));
    (old_link, new_link)
}

pub fn pop_context(old_link: Option<RcContext>) {
    let old_current = set_link(old_link);
    set_current(old_current);
}

pub fn get_link() -> Option<RcContext> {
    read_key(&LINK)
}

#[inline]
pub fn set_link(new: Option<RcContext>) -> Option<RcContext> {
    set_key(&LINK, new)
}
#[inline]
pub fn read_current() -> Option<RcContext> {
    read_key(&CURRENT)
}
#[inline]
pub fn set_current(new: Option<RcContext>) -> Option<RcContext> {
    set_key(&CURRENT, new)
}
#[inline]
fn read_key(key: &'static LocalKey<Cell<*mut Context>>) -> Option<RcContext> {
    let cx = key.get();
    let cx = NonNull::new(cx)?;
    let cx = RcContext(cx);
    let out = cx.clone();
    forget(cx);
    Some(out)
}
#[inline]
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
