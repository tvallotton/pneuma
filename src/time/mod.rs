use pneuma::reactor::imp::op;
use std::io;
pub use std::time::*;

pub fn sleep(dur: Duration) -> io::Result<()> {
    op::sleep(dur)
}
