use std::cell::Cell;
use std::{net::Shutdown, rc::Rc};
// use pneuma::reactor::Reactor;
// use pneuma::thread::JoinHandle;
use executor::Executor;

use pneuma::thread::RcContext;

// mod config;
mod executor;

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
}

impl std::ops::Deref for Runtime {
    type Target = InnerRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn current() -> Runtime {
    pneuma::thread::current().0.runtime.clone()
}
