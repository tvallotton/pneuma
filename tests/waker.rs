use pneuma::thread::{park, spawn, yield_now};
use std::task::Waker;

#[test]
fn single_threaded_wakup() {}

#[ignore]
#[test]
fn cross_thread_wakup() {
    let handle = spawn(|| {
        dbg!();
        park();
    });
    yield_now();
    let waker: Waker = handle.thread().clone().into();
    std::thread::spawn(move || {
        dbg!();
        waker.wake();

        dbg!();
    });

    handle.join();
}
