use pneuma::thread;
use std::cell::Cell;
use std::rc::Rc;

fn counter_assert(counter: &Cell<i32>, prediction: i32) {
    assert_eq!(counter.get(), prediction);
    counter.set(counter.get() + 1);
}

#[test]
fn yield_now() {
    let counter = Rc::new(Cell::new(1));
    let counter_ = counter.clone();

    let handle = pneuma::thread::spawn(move || {
        counter_assert(&counter_, 2);
        pneuma::thread::yield_now();
        counter_assert(&counter_, 4);
        pneuma::thread::yield_now();
        counter_assert(&counter_, 5);
        122
    });

    counter_assert(&counter, 1);
    pneuma::thread::yield_now();
    counter_assert(&counter, 3);
    assert_eq!(handle.join(), 122);
    counter_assert(&counter, 6);
}

// #[test]
// fn spawn() {
//     let handle = pneuma::thread::Builder::new()
//         .spawn(|| {
//             println!("task: init");
//             pneuma::thread::yield_now();
//             println!("task: middle ");
//             pneuma::thread::yield_now();
//             println!("task: exiting");
//             122
//         })
//         .unwrap();

//     println!("main: spawned");
//     pneuma::thread::yield_now();
//     println!("main: yielded");
//     assert_eq!(handle.join(), 122);
//     println!("main: finished");
// }
