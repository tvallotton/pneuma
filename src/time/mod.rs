use std::{io, time::Duration};

use pneuma::reactor::imp::op;

pub fn sleep(dur: Duration) -> io::Result<()> {
    op::sleep(dur)
}
