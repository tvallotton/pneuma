use std::{thread, time::Duration};

use pneuma::uthread::{self, spawn};

const TIMEOUT_MILLIS: u64 = 20;

fn stealer_thread() {
    // kernel thread has nothing to do now, it will try to steal tasks.
    pneuma::time::sleep(Duration::from_millis(TIMEOUT_MILLIS));
    dbg!();
}

fn stolen_thread() {
    pneuma::uthread::spawn(|| {
        // block the worker thread so the os_thread can be stolen
        std::thread::sleep(Duration::from_millis(TIMEOUT_MILLIS));
        dbg!();
    });
    // make sure this os_thread is always ready so it can be stolen
    for i in 0..100 {
        println!("{i} {:?}", std::thread::current().name());
        pneuma::uthread::yield_now();
    }
    dbg!();
}

#[test]
fn no_os_thread_workstealing() {
    let stealer_handle = thread::Builder::new()
        .name("stealer".into())
        .spawn(stealer_thread)
        .unwrap();
    let stolen_handle = thread::Builder::new()
        .name("stolen".into())
        .spawn(stolen_thread)
        .unwrap();

    stolen_handle.join().unwrap();
    stealer_handle.join().unwrap()
}
