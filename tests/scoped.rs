use pneuma::uthread::yield_now;

#[test]
fn access_local_variable() {
    use pneuma::uthread;

    let variable = vec![1, 2, 34];
    uthread::scope(|s| {
        let t = s.spawn(|| {
            println!("hello {variable:?}");
        });

        println!("thread id: {:?}", t.thread().id());
    });
}
