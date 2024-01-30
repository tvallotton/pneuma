use pneuma::thread::Thread;
use std::{
    io::Write,
    mem::{transmute, MaybeUninit},
    os::fd::AsRawFd,
    ptr::drop_in_place,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

use crate::thread::ReprContext;

use super::thin_waker::ThinWaker;

pub struct SendThread(pub Thread);

impl From<Thread> for Waker {
    fn from(thread: Thread) -> Self {
        let waker = ThinWaker::from(thread);
        let waker = RawWaker::new(unsafe { transmute(waker) }, VTABLE);
        unsafe { Waker::from_raw(waker) }
    }
}

pub unsafe fn raw_waker(data: Thread) -> RawWaker {
    RawWaker::new(transmute(data), VTABLE)
}

pub unsafe fn clone(waker: *const ()) -> RawWaker {
    let thinwaker: &ThinWaker = transmute(&waker);
    std::mem::forget(thinwaker.clone());
    RawWaker::new(waker, VTABLE)
}

pub unsafe fn wake(waker: *const ()) {
    let thinwaker: ThinWaker = transmute(waker);
    thinwaker.wake_by_ref();
}

pub unsafe fn wake_by_ref(waker: *const ()) {
    let thinwaker: &ThinWaker = transmute(&waker);
    thinwaker.wake_by_ref();
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
