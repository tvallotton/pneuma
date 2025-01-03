use std::time::{Duration, Instant};

use pneuma::{
    time::sleep,
    uthread::{self, yield_now},
};

#[test]
fn smoke_sleep() {
    let dur = Duration::from_millis(50);
    let time = Instant::now();
    sleep(dur);
    assert!(time.elapsed() >= dur);
}

#[test]
fn sleep_even_if_unparked() {
    let dur = Duration::from_millis(50);
    let time = Instant::now();
    let handle = uthread::spawn(move || {
        sleep(dur);
    });
    yield_now();
    handle.thread().unpark();
    handle.thread().unpark();
    handle.join();
    assert!(time.elapsed() >= dur);
}
