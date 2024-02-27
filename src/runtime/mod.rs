use std::cell::Cell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;

// use pneuma::reactor::Reactor;
use executor::Executor;
pub use globals::current;
use pneuma::thread::{Builder, JoinHandle};

use pneuma::reactor::Reactor;

use self::singal_stack::SignalStack;
pub(crate) use shared_queue::SharedQueue;

// mod config;
mod executor;
mod globals;
mod shared_queue;
mod singal_stack;

#[derive(Clone)]
pub(crate) struct Runtime(Rc<InnerRuntime>);

#[allow(unused)]
pub(crate) struct InnerRuntime {
    polls: Cell<u64>,
    pub executor: Executor,
    pub reactor: Reactor,
    pub shared_queue: Arc<SharedQueue>,

    pub signal_stack: SignalStack,
}

impl Runtime {
    #[rustfmt::skip]
    pub(crate) fn new() -> io::Result<Self> {
        let polls         = Cell::new(0);
        let signal_stack = SignalStack::new()?;
        let shared_queue  = SharedQueue::new()?;
        let reactor       = Reactor::new(shared_queue.clone())?;
        let executor      = Executor::new(shared_queue.clone());

        let inner = InnerRuntime {
            executor,
            polls,
            reactor,
            shared_queue,
            signal_stack,
        };

        let rt = Runtime(Rc::new(inner));

        Ok(rt)
    }

    pub(crate) fn shutdown(self) {
        while self.executor.total_threads() > 1 {
            self.executor.unpark_all();
            pneuma::thread::yield_now();
        }
    }

    #[inline]
    pub fn poll_reactor(&self) -> io::Result<()> {
        let mut queue = self.shared_queue.queue.lock().unwrap();
        dbg!(queue.is_empty());
        while let Some(waker) = queue.pop_front() {
            dbg!();
            unsafe { waker.local_wake() };
        }
        drop(queue);

        if self.executor.is_empty() {
            self.shared_queue.sleep(|| self.reactor.submit_and_wait())?;
        } else {
            self.reactor.submit_and_yield()?;
        }
        dbg!(self.executor.is_empty());
        Ok(())
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

    pub(crate) fn spawn<T, F>(&self, f: F, builder: Builder) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let sq = self.shared_queue.clone();
        self.executor.spawn(f, sq, builder)
    }
}

impl std::ops::Deref for Runtime {
    type Target = InnerRuntime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
