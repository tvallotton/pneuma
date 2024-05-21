use pneuma::uthread::{spawn, yield_now};

#[test]
fn a_smoke_test() {
    dbg!("a_smoke_test 0");
    let handle = spawn(|| {
        pneuma::uthread::yield_now();
    });
    pneuma::uthread::yield_now();
    handle.join();
}

#[test]
fn leak() {
    spawn(|| {
        println!("asd");
    });
}

#[test]
fn a_orphan() {
    let handle = spawn(|| pneuma::uthread::park().unwrap());
    yield_now();
    drop(handle);
    yield_now();
}
