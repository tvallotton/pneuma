use pneuma::{sync::Mutex, uthread::yield_now};
#[ignore = "mutex is unsound, it causes deadlocks"]
#[test]
fn mutex_hold_across_yield_point() {
    static MUTEX: Mutex<i32> = Mutex::new(0);

    let handle = pneuma::uthread::spawn(|| {
        let mut guard = MUTEX.lock();
        yield_now();
        *guard += 1;
    });

    handle.join();
    assert_eq!(*MUTEX.lock(), 1);
}
#[ignore = "mutex is unsound, it causes deadlocks"]
#[test]
fn mutex_contention() {
    use std::sync::Arc;
    let mutex = Arc::new(Mutex::new(0));

    let mut handles = vec![];
    for i in 0..100 {
        let mutex = mutex.clone();
        let handle = pneuma::uthread::spawn(move || {
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
#[ignore = "mutex is unsound, it causes deadlocks"]
#[test]
fn mutex_try_lock() {
    static MUTEX: Mutex<i32> = Mutex::new(0);

    let guard = MUTEX.try_lock();
    format!("{}", &MUTEX.try_lock().unwrap_err().clone());
    drop(guard)
}

#[test]
fn mutex_get_mut() {
    let mut mutex = Mutex::new(11);
    assert_eq!(*mutex.get_mut(), 11);
}
#[ignore = "mutex is unsound, it causes deadlocks"]
#[test]
fn mutex_into_inner() {
    let mutex = Mutex::new(11);
    assert_eq!(mutex.into_inner(), 11);
}

#[test]
fn mutex_debug_and_display() {
    let mutex = Mutex::new("hello world");
    let guard = mutex.lock();

    assert!(format!("{mutex:?}").contains("<locked>"));
    assert_eq!(format!("{:?}", guard), format!("{:?}", *guard));
    assert_eq!(format!("{}", guard), format!("{}", *guard));
    drop(guard);
    assert!(format!("{mutex:?}").contains("hello world"));
}

#[test]
fn mutex_default() {
    assert_eq!(
        Mutex::<String>::default().into_inner(),
        Mutex::new(String::default()).into_inner()
    );
}

#[test]
fn mutex_poison() {
    static MUTEX: Mutex<i32> = Mutex::new(0);
    pneuma::uthread::spawn(|| {
        let _guard = MUTEX.lock();
        panic!();
    })
    .try_join()
    .unwrap_err();

    MUTEX.lock();
}
#[ignore = "mutex is unsound, it causes deadlocks"]
#[test]
fn mutex_try_lock_and_lock() {
    static MUTEX: Mutex<i32> = Mutex::new(0);

    pneuma::uthread::scope(|s| {
        for _ in 0..100 {
            s.spawn(|| {
                let Ok(mut t) = MUTEX.try_lock() else {
                    *MUTEX.lock() += 1;
                    return;
                };
                *t += 1;
            });
        }
    });

    assert_eq!(100, *MUTEX.lock());
}
