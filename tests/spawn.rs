use std::{cell::Cell, rc::Rc};

use pneuma::thread::{self, current};

#[test]
fn current_for_os_thread() {
    assert_eq!(thread::current().name(), std::thread::current().name());
}

#[test]
fn current_for_green_thread() {
    thread::Builder::new()
        .name("test".into())
        .spawn(|| {
            assert_eq!(thread::current().name().unwrap(), "test");
        })
        .unwrap()
        .join();
}

#[test]
fn yield_now() {
    let count = Rc::new(Cell::new(1));
    let handle = thread::spawn({
        let count = count.clone();
        move || {
            let n = count.get();
            println!("1");
            thread::yield_now();
            println!("3");
            assert_ne!(n, count.get());
        }
    });
    println!("0");
    thread::yield_now();
    println!("2");
    count.set(count.get() + 1);
    handle.join();
}
