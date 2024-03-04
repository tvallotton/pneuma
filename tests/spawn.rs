use std::cell::Cell;
use std::rc::Rc;

fn counter_assert(counter: &Cell<i32>, prediction: i32) {
    assert_eq!(counter.get(), prediction);
    counter.set(counter.get() + 1);
}

#[test]
fn yield_now() {
    let counter = Rc::new(Cell::new(1));
    let counter_ = counter.clone();

    let handle = pneuma::thread::spawn(move || {
        counter_assert(&counter_, 2);
        pneuma::thread::yield_now();

        counter_assert(&counter_, 4);
        pneuma::thread::yield_now();
        counter_assert(&counter_, 5);
        122
    });

    counter_assert(&counter, 1);
    pneuma::thread::current().unpark();
    pneuma::thread::yield_now();
    counter_assert(&counter, 3);
    dbg!();
    assert_eq!(handle.join(), 122);
    counter_assert(&counter, 6);
}

#[test]
fn panic_from_green_thread() {
    let handle = pneuma::thread::spawn(move || panic!("test panic"));
    println!("handle-1");
    assert!(handle.try_join().is_err());
}

#[test]
fn dangling() {
    pneuma::thread::spawn(move || {
        pneuma::thread::yield_now();
    });
    pneuma::thread::yield_now();
}

/// this leaks because libunwind caches memory
#[test]
fn backtrace() {
    pneuma::thread::spawn(move || {
        println!("{}", std::backtrace::Backtrace::force_capture());
    })
    .join();
}

#[test]
fn name() {
    pneuma::thread::Builder::new()
        .name("test-thread".into())
        .spawn(|| {
            assert_eq!(pneuma::thread::current().name(), Some("test-thread"));
        })
        .unwrap()
        .join();
}

#[test]
fn is_cancelled() {
    let main = pneuma::thread::current();
    pneuma::thread::spawn(move || {
        while dbg!(!pneuma::thread::is_cancelled()) {
            dbg!(&main);
            pneuma::thread::yield_now();
        }
    });

    // changing this to ::yield_now() causes a segfault
    pneuma::thread::yield_now()
}

#[test]
#[should_panic]
fn panic_with_unjoined_task() {
    pneuma::thread::spawn(|| {
        pneuma::thread::yield_now();
    });
    pneuma::thread::yield_now();
    panic!();
}
