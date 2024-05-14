use std::{
    io::{self, Error},
    sync::atomic::{AtomicU64, Ordering::Release},
};

use crate::{reactor::Reactor, sys::signal_stack::SignalStack};
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

    pub fn park(&self) -> io::Result<()> {
        self.increment_tick()?;

        // NOTE: we might never return
        // better not leave any variables undropped
        let res = self.executor.context_switch();

        if res.is_err() {
            self.reactor.submit_and_wait()?;
            self.executor.context_switch().unwrap();
        }

        Ok(())
    }

    pub fn increment_tick(&self) -> io::Result<()> {
        let prev = self.tick.fetch_add(1, Release);
        if prev % 61 == 0 {
            self.reactor.submit_and_yield()?;
        }

        Ok(())
    }
}
