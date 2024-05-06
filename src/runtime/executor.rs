use crossbeam_deque::{Stealer, Worker};

use std::{collections::VecDeque, mem::replace, sync::Mutex};
use thread_local::ThreadLocal;

use crate::{sys::stack::Stack, uthread::UThread};

const MAX_WORK_PER_WORKER: usize = 16;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub worker: ThreadLocal<crossbeam_deque::Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub injector: crossbeam_deque::Injector<UThread>,

    pub all: Mutex<VecDeque<UThread>>,

    pub _unused_stacks: Mutex<Vec<Stack>>,
}

impl Executor {
    pub fn context_switch(&self) -> Result<(), ()> {
        let new = self.pop().ok_or(())?;

        let old = self.set_current(new.clone());

        old.cx.switch_to(new.cx);

        Ok(())
    }

    pub(crate) fn current(&self) -> &Mutex<UThread> {
        self.current.get_or(|| Mutex::new(UThread::for_os_thread()))
    }

    fn set_current(&self, with: UThread) -> UThread {
        let mut current = self.current().lock().unwrap();
        replace(&mut *current, with)
    }

    fn pop(&self) -> Option<UThread> {
        let worker = self.worker();

        let thread = worker.pop();

        if thread.is_some() {
            return thread;
        }

        let thread = self
            .injector
            .steal_batch_with_limit_and_pop(worker, MAX_WORK_PER_WORKER)
            .success();

        if thread.is_some() {
            return thread;
        }

        let mut stealers: Vec<_> = self.stealers.iter().collect();

        fastrand::shuffle(&mut stealers);

        return stealers
            .iter()
            .map(|s| s.steal_batch_and_pop(worker))
            .filter_map(|steal| steal.success())
            .next();
    }

    pub fn worker(&self) -> &Worker<UThread> {
        self.worker.get_or(|| {
            let worker = crossbeam_deque::Worker::new_fifo();
            self.stealers.get_or(|| worker.stealer());
            worker
        })
    }

    pub fn push(&self, thread: UThread) {
        if fastrand::u8(0..4) == 0 {
            return self.injector.push(thread);
        }

        if let Some(worker) = self.worker.get() {
            if worker.len() < MAX_WORK_PER_WORKER {
                return worker.push(thread);
            }
        }
        self.injector.push(thread)
    }
}
