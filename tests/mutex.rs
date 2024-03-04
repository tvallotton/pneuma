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
    static MUTEX: Mutex<i32> = Mutex::new(0);

    let mut handles = vec![];
    for _ in 0..100 {
        let handle = pneuma::thread::spawn(move || {
            let mut guard = MUTEX.lock();
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

    assert_eq!(*MUTEX.lock(), 100);
}
