use pneuma::thread::Thread;
use std::{
    mem::transmute,
    ptr::drop_in_place,
    task::{RawWaker, RawWakerVTable, Waker},
};

use pneuma::thread::ReprContext;

use super::thin_waker::ThinWaker;

impl From<Thread> for Waker {
    fn from(thread: Thread) -> Self {
        let waker = ThinWaker::from(thread);
        let waker = RawWaker::new(unsafe { transmute(waker) }, VTABLE);
        unsafe { Waker::from_raw(waker) }
    }
}

pub unsafe fn clone(waker: *const ()) -> RawWaker {
    let thinwaker: &ThinWaker = transmute(&waker);
    std::mem::forget(thinwaker.clone());
    RawWaker::new(waker, VTABLE)
}

pub unsafe fn wake(waker: *const ()) {
    let thinwaker: ThinWaker = transmute(waker);
    thinwaker.wake();
}

pub unsafe fn wake_by_ref(waker: *const ()) {
    let thinwaker: &ThinWaker = transmute(&waker);
    thinwaker.wake_by_ref();
}

pub unsafe fn drop(waker: *const ()) {
    let _: ThinWaker = transmute(waker);
}

static VTABLE: &RawWakerVTable = &RawWakerVTable::new(clone, wake, wake_by_ref, drop);
