use std::{rc::Rc, time::Duration};

use pneuma::{sync::Mutex, thread::yield_now};

#[test]
fn hold_across_yield_point() {
    static MUTEX: Mutex<i32> = Mutex::new(0);

    let handle = pneuma::thread::spawn(|| {
        let mut guard = MUTEX.lock();
        yield_now();
        *guard += 1;
    });

    handle.join();
    assert_eq!(*MUTEX.lock(), 1);
}

#[test]
fn contention() {
    use std::rc::Rc;
    let mutex = Rc::new(Mutex::new(0));

    let mut handles = vec![];
    for _ in 0..100 {
        let mutex = mutex.clone();
        let handle = pneuma::thread::spawn(move || {
            let mut guard = mutex.lock();
            for _ in 0..10 {
                yield_now();
            }
            *guard += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join()
    }

    assert_eq!(*mutex.lock(), 100);
}

#[test]
fn try_lock() {
    static MUTEX: Mutex<i32> = Mutex::new(0);

    let guard = MUTEX.try_lock();
    format!("{}", &MUTEX.try_lock().unwrap_err().clone());
    drop(guard)
}

#[test]
fn get_mut() {
    let mut mutex = Mutex::new(11);
    assert_eq!(*mutex.get_mut(), 11);
}

#[test]
fn into_inner() {
    let mutex = Mutex::new(11);
    assert_eq!(mutex.into_inner(), 11);
}

#[test]
fn debug_and_display() {
    let mutex = Mutex::new("hello world");
    let guard = mutex.lock();

    assert!(format!("{mutex:?}").contains("<locked>"));
    assert_eq!(format!("{:?}", guard), format!("{:?}", *guard));
    assert_eq!(format!("{}", guard), format!("{}", *guard));
    drop(guard);
    assert!(format!("{mutex:?}").contains("hello world"));
}

#[test]
fn default() {
    assert_eq!(
        Mutex::<String>::default().into_inner(),
        Mutex::new(String::default()).into_inner()
    );
}

#[test]
fn poison() {
    static MUTEX: Mutex<i32> = Mutex::new(0);
    pneuma::thread::spawn(|| {
        let _guard = MUTEX.lock();
        panic!();
    })
    .try_join();

    MUTEX.lock();
}
