use std::{
    io::{self},
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
    pub(crate) _signal_stack: SignalStack,
}

impl Runtime {
    fn new() -> io::Result<Self> {
        let tick = AtomicU64::new(0);
        let executor = Executor::default();
        let reactor = Reactor::new()?;
        let _signal_stack = SignalStack::new()?;

        Ok(Runtime {
            tick,
            executor,
            reactor,
            _signal_stack,
        })
    }

    pub fn park(&self) -> io::Result<()> {
        self.increment_tick()?;

        // NOTE: we might never return
        // better not leave any variables undropped
        let res = self.executor.context_switch();

        if res.is_err() {
            self.reactor.submit_and_wait()?;
            self.executor.context_switch().ok();
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
