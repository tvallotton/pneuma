use std::time::{Duration, Instant};

use pneuma::time::sleep;

#[test]
fn smoke_sleep() {
    let time = Instant::now();
    sleep(Duration::from_millis(500));
    dbg!(time.elapsed());
}
