use std::io;

use mio::Poll;

pub struct Reactor {
    poll: Poll,
}

impl Reactor {
    pub fn new() -> io::Result<Reactor> {
        let poll = Poll::new()?;
        Ok(Reactor { poll })
    }
}
