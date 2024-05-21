use std::{
    future::Future,
    mem::transmute,
    pin::pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::uthread::UThread;

/// Awaits an async function
pub fn wait<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut future = pin!(future);
    let waker = pneuma::uthread::current().into_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(payload) = future.as_mut().poll(&mut cx) {
            return payload;
        };
        pneuma::uthread::park().unwrap();
    }
}

impl UThread {
    pub fn into_waker(self) -> Waker {
        unsafe { Waker::from_raw(self.into_raw_waker()) }
    }

    unsafe fn into_raw_waker(self) -> RawWaker {
        RawWaker::new(transmute(self), &VTABLE)
    }
}

const VTABLE: RawWakerVTable = unsafe {
    RawWakerVTable::new(
        |data| {
            let uthread: &UThread = transmute(&data);
            uthread.clone().into_raw_waker()
        },
        |data| {
            let uthread: UThread = transmute(data);
            uthread.unpark();
        },
        |data| {
            let uthread: &UThread = transmute(&data);
            uthread.unpark();
        },
        |data| {
            let _: UThread = transmute(data);
        },
    )
};
