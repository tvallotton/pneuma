use std::{cell::OnceCell, iter::Once, sync::OnceLock};

use pneuma::uthread::{spawn, yield_now};

#[test]
fn a_smoke_test() {
    let handle = spawn(|| {
        dbg!(2);
        pneuma::uthread::yield_now();
        dbg!(3);
        pneuma::uthread::yield_now();
        dbg!(5);

        pneuma::uthread::yield_now();
        dbg!(6)
    });

    dbg!(1);
    pneuma::uthread::yield_now();
    dbg!(4);
    handle.join();
    dbg!(7);
}



#[test]
fn a_leak() {
    spawn(|| {
        println!("asd");
    });
    
}

#[test]
fn a_orphan() {
    dbg!("a");
    let handle = spawn(|| pneuma::uthread::park().unwrap());
    yield_now();
    drop(handle);
    yield_now();
}
