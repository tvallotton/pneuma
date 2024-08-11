#[test]
fn block() {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = pneuma::uthread::spawn(move || {
        dbg!(std::thread::current().id());
        pneuma::uthread::block(|| {
            dbg!(std::thread::current().id());
            assert_eq!(rx.recv().unwrap(), 99);
        });
        dbg!();
    });
    pneuma::uthread::spawn(move || {
        dbg!(std::thread::current().id());
        pneuma::uthread::yield_now();
        dbg!(std::thread::current().id());
        tx.send(99).unwrap();
        handle.join();
    })
    .join();
}
