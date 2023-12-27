use std::{cell::RefCell, collections::VecDeque};

use crate::thread::{JoinHandle, Task};

pub struct Executor {
    run_queue: RefCell<VecDeque<Task>>,
}

impl Executor {
    pub fn spawn<'a, F, T>(&self, f: F) -> JoinHandle<'a, T>
    where
        F: FnOnce() -> T + 'a,
    {
      todo!()
    }
}
