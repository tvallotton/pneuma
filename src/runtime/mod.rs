use pneuma::thread::park;
use std::cell::Cell;
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
    shutdown: Cell<bool>,
    polls: Cell<usize>,
    pub executor: Executor,
    // reactor: Reactor,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let executor = Executor::new();
        let shutdown = Cell::new(false);
        let polls = Cell::new(0);
        Runtime(Rc::new(InnerRuntime {
            executor,
            shutdown,
            polls,
        }))
    }

    pub(crate) fn shutdown(self) {
        self.shutdown.set(true);

        while !self.executor.is_empty() {
            park()
        }
    }

    // /// Switches to the next
    // pub fn switch(&self) -> RcContext {
    //     let polls = (self.polls.get() + 1) % 61;
    //     self.polls.set(polls);
    //     let option = self.executor.run_queue.borrow_mut().pop_front();
    //     if option.is_none() {
    //         self.reactor
    //     }

    //     if polls == 0 && self.executor.is_empty() {
    //         self.poll_reactor();
    //     }

    //     if let Some(cx) ={
    //         return cx;
    //     }

    // }

    #[inline]
    pub fn poll_reactor(&self) {
        // if self.executor.is_empty() {
        //     self.reactor.poll_and_wait();
        // } else {
        //     self.reactor.poll_and_yield();
        // }
    }

    /// Periodically poll the reactor
    pub fn poll(&self) {
        let polls = (self.polls.get() + 1) % 61;
        self.polls.set(polls);
        if polls == 0 {
            self.poll_reactor()
        }
    }

    pub fn park(&self) {
        self.poll();
        if let Some(next) = self.executor.pop() {
            return self.executor.switch_to(next);
        }
        self.poll_reactor();
    }
}

impl std::ops::Deref for Runtime {
    type Target = InnerRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {}
}
