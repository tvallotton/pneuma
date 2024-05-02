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
        let signal_stack = SignalStack::new();

        todo!()
    }

    fn park(&self) -> io::Result<()> {
        todo!()
    }
}
