use crate::task::JoinHandle;
use crate::task::{RcContext, Task};
use std::{cell::RefCell, collections::VecDeque};

pub struct Executor {
    run_queue: RefCell<VecDeque<RcContext>>,
}

impl Executor {
    pub fn spawn<T, F>(&self, size: usize, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        JoinHandle(Task::new(size, f))
    }
}
