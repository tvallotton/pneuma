pub use std::time::Duration;

use pneuma::reactor::op;

pub fn sleep(dur: Duration) {
    op::sleep(dur).unwrap();
}
