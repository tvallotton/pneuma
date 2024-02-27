use std::{io, mem::transmute, ptr::Thin, time::Duration};

use mio::{
    event::{Event, Events},
    Poll, Waker,
};

use crate::future::thin_waker::ThinWaker;

pub struct Reactor {
    poll: Poll,
    events: Events,
}

impl Reactor {
    pub fn new() -> io::Result<Reactor> {
        let poll = Poll::new()?;
        let events = Events::with_capacity(256);
        Ok(Reactor { poll, events })
    }

    pub fn push(&self, event: Event) -> io::Result<()> {
        // self.poll.poll(events, timeout)

        todo!()
    }

    pub fn submit_and_yield(&mut self) -> io::Result<()> {
        self.poll.poll(&mut self.events, Some(Duration::ZERO))?;
        self.wake_tasks();
        Ok(())
    }

    pub fn submit_and_wait(&mut self) -> io::Result<()> {
        self.poll.poll(&mut self.events, None)?;
        self.wake_tasks();
        Ok(())
    }

    fn wake_tasks(&self) {
        for event in &self.events {
            let waker: ThinWaker = unsafe { transmute(event.token().0) };
            waker.wake();
        }
    }
}
