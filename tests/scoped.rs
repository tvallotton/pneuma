use pneuma::uthread;

#[test]
fn access_local_variable() {
    let variable = vec![1, 2, 34];
    uthread::scope(|s| {
        let t = s.spawn(|| {
            println!("hello {variable:?}");
        });

        println!("thread id: {:?}", t.thread().id());
    });
}
#[should_panic]
#[test]
fn unhandled_panic_drop_before_panic() {
    uthread::scope(|s| {
        let t = s.spawn(|| {
            panic!("oh no");
        });

        drop(t);
    });
}

#[should_panic]
#[test]
fn unhandled_panic_drop_after_panic() {
    let parent = uthread::current();

    uthread::scope(|s| {
        let t = s.spawn(|| {
            parent.unpark();
            panic!("oh no");
        });

        uthread::park();

        while !t.is_finished() {
            uthread::yield_now();
        }

        drop(t);
    });
}

#[test]
fn handled_panic_join_before_panic() {
    uthread::scope(|s| {
        let t = s.spawn(|| {
            panic!("oh no");
        });

        t.try_join().ok();
    });
}

#[test]
fn handled_panic_join_after_panic() {
    let parent = uthread::current();

    uthread::scope(|s| {
        let t = s.spawn(|| {
            parent.unpark();
            panic!("oh no");
        });

        uthread::park();

        while !t.is_finished() {
            uthread::yield_now();
        }

        t.try_join().ok();
    });
}

#[should_panic]
#[test]
fn propagated_panic() {
    uthread::scope(|s| {
        let t = s.spawn(|| {
            panic!("oh no");
        });

        t.join();
    });
}
