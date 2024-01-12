use std::cell::Cell;
use std::io;
use std::rc::Rc;
// use pneuma::reactor::Reactor;
// use pneuma::thread::JoinHandle;
use executor::Executor;
pub use globals::current;

// mod config;
mod executor;
mod globals;

#[derive(Clone)]
pub(crate) struct Runtime(Rc<InnerRuntime>);

pub(crate) struct InnerRuntime {
    polls: Cell<usize>,
    pub executor: Executor,
    #[cfg(feature = "io")]
    pub reactor: pneuma::reactor::Reactor,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let executor = Executor::new();
        let polls = Cell::new(0);
        #[cfg(feature = "io")]
        let reactor = pneuma::reactor::Reactor::new();
        Runtime(Rc::new(InnerRuntime {
            executor,
            polls,
            #[cfg(feature = "io")]
            reactor,
        }))
    }

    pub(crate) fn shutdown(self) {
        while self.executor.total_threads() > 1 {
            self.executor.unpark_all();
            pneuma::thread::yield_now();
        }
    }

    #[inline]
    pub fn poll_reactor(&self) -> io::Result<()> {
        if self.executor.is_empty() {
            self.reactor.submit_and_wait()
        } else {
            self.reactor.submit_and_yield()
        }
    }

    /// Periodically poll the reactor
    pub fn poll(&self) -> io::Result<()> {
        let polls = (self.polls.get() + 1) % 61;
        self.polls.set(polls);
        if polls == 0 || self.executor.is_empty() {
            return self.poll_reactor();
        }
        Ok(())
    }

    pub fn park(self) {
        self.poll().unwrap();
        if let Some(next) = self.executor.pop() {
            self.executor.switch_to(next);
        }
    }
}

impl std::ops::Deref for Runtime {
    type Target = InnerRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
