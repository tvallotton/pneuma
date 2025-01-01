// use std::time::{Duration, Instant};

// use pneuma::uthread::{self, yield_now};
// #[ignore = "failing"]
// #[test]
// fn block() {
//     let time = Instant::now();

//     // pneuma::uthread::current();
//     let handle = pneuma::uthread::spawn(move || {
//         pneuma::uthread::block(|| {
//             pneuma::time::sleep(std::time::Duration::from_secs(50));
//         });

//         assert!(Duration::from_millis(50) < time.elapsed())
//     });

//     pneuma::uthread::spawn(move || {
//         uthread::yield_now();
//         assert!(time.elapsed() < Duration::from_millis(50))
//     })
//     .join();
//     // handle.join();
//     ();
// }
