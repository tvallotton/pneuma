use pneuma::task::Thread;
use pneuma::task::{JoinHandle, Stack};
use std::io;
use std::{cell::RefCell, collections::VecDeque};

pub(crate) struct Executor {
    pub run_queue: RefCell<VecDeque<Thread>>,
    pub unused_stacks: RefCell<Vec<Stack>>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            run_queue: RefCell::default(),
            unused_stacks: RefCell::default(),
        }
    }

    pub fn push(&self, thread: Thread) {
        self.run_queue.borrow_mut().push_back(thread);
    }

    pub fn pop(&self) -> Option<Thread> {
        self.run_queue.borrow_mut().pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.run_queue.borrow().is_empty()
    }
}
