use std::thread::scope;

use pneuma::uthread::spawn;

#[test]
fn smoke_mpmc() {
    const ITERATIONS: u32 = 100;
    let (ref sender, receiver) = pneuma::sync::mpmc::channel();

    std::thread::scope(|s| {
        for i in 0..ITERATIONS {
            s.spawn(move || sender.send(i));
        }
    });

    for _ in 0..ITERATIONS {
        receiver.recv().unwrap();
    }
}
