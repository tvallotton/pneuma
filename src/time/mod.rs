pub use std::time::Duration;

use crate::reactor::op;

pub fn sleep(dur: Duration) {
    op::sleep(dur).unwrap();
}
