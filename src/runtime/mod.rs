use std::{
    io,
    os::fd::AsRawFd,
    sync::{atomic::AtomicU64, Mutex},
};

use crate::{
    reactor::{self, Reactor},
    sys::signal_stack::{self, SignalStack},
};
pub(crate) use globals::current;

use executor::Executor;
mod executor;
mod globals;

pub(crate) struct Runtime {
    pub(crate) tick: AtomicU64,
    pub(crate) executor: Executor,
    pub(crate) reactor: Reactor,
    pub(crate) signal_stack: SignalStack,
}

impl Runtime {
    fn new() -> io::Result<Self> {
        let tick = AtomicU64::new(0);
        let executor = Executor::default();
        let reactor = Reactor::new()?;
        let signal_stack = SignalStack::new()?;

        Ok(Runtime {
            tick,
            executor,
            reactor,
            signal_stack,
        })
    }
    
    fn park(&self) -> io::Result<()> {
        self.increment_tick()?;

        let res = self.executor.yield_to();
        if res.err() {
            
        }

        todo!()
    }

    pub fn increment_tick(&self) -> io::Result<()> {
        let prev = self.tick.fetch_add(1, Release);
        if prev % 61 == 0 {
            self.reactor.submit_and_yield()?;
        }

        if prev % 512 == 0 {
            self.executor.even_queues(); 
        }
        Ok(())
    }
}
