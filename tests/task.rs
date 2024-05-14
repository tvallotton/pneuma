use pneuma::uthread::spawn;

#[test]
fn smoke_test() {
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
