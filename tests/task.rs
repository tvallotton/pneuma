use pneuma::uthread::spawn;

#[test]
fn smoke_test() {
    let handle = spawn(|| {
        dbg!();
        pneuma::uthread::yield_now();
        dbg!();
    });

    dbg!();
    pneuma::uthread::park().unwrap();
    dbg!();
    handle.join();
}
