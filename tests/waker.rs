use pneuma::thread::{park, spawn, yield_now};
use std::task::Waker;

#[test]
fn single_threaded_wakup() {
    let handle = pneuma::thread::current();
    spawn(|| {
        let waker: Waker = handle.into();
        waker.wake();
    });
    park();
}

#[test]
fn cross_thread_wakup() {
    let waker: Waker = pneuma::thread::current().clone().into();
    std::thread::spawn(move || {
        waker.clone().wake_by_ref();
        waker.wake();
    });
    park()
}

#[test]
fn cross_thread_waker_drop_after_gthread_exit() {
    
}