use std::time::{Duration, Instant};

use pneuma::time::sleep;

#[test]
fn smoke_sleep() {
    let dur = Duration::from_millis(50);
    let time = Instant::now();
    sleep(dur);
    assert!(time.elapsed() >= dur);
}
