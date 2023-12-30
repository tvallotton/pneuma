use crate::task::RcContext;
use crate::task::{JoinHandle, Stack};
use std::io;
use std::{cell::RefCell, collections::VecDeque};

pub(crate) struct Executor {
    pub run_queue: RefCell<VecDeque<RcContext>>,
    pub unused_stacks: RefCell<Vec<Stack>>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            run_queue: RefCell::default(),
            unused_stacks: RefCell::default(),
        }
    }

    pub fn spawn<T, F>(&self, size: usize, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        JoinHandle::new(size, f)
    }

    pub fn is_empty(&self) -> bool {
        self.run_queue.borrow().is_empty()
    }
}
