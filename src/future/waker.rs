use pneuma::thread::Thread;
use std::{
    mem::transmute,
    ptr::drop_in_place,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::thread::ReprContext;

impl From<Thread> for Waker {
    fn from(value: Thread) -> Self {
        // value
        todo!()
    }
}

pub fn raw_waker() -> RawWaker {
    todo!()
}

pub unsafe fn clone(waker: *const ()) -> RawWaker {
    let cx: &ReprContext = transmute(waker);
    let old_count = cx
        .atomic_refcount
        .fetch_add(1, std::sync::atomic::Ordering::Acquire);

    RawWaker::new(waker, VTABLE)
}

pub unsafe fn wake(waker: *const ()) {
    todo!()
}

pub unsafe fn wake_by_ref(waker: *const ()) {
    todo!()
}

pub unsafe fn drop(waker: *const ()) {
    let cx: &ReprContext = transmute(waker);
    let old_count = cx
        .atomic_refcount
        .fetch_sub(1, std::sync::atomic::Ordering::Release);

    if old_count == 1 {
        let layout = cx.layout;
        drop_in_place(waker as *mut ReprContext);
        unsafe { std::alloc::dealloc(waker as _, layout) };
    }
}

static VTABLE: &RawWakerVTable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
