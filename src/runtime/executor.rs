use crossbeam_deque::{Steal, Stealer, Worker};

use std::{
    collections::VecDeque,
    mem::replace,
    sync::{atomic::Ordering, Mutex},
};
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
        new.cx.is_queued.store(false, Ordering::Relaxed);

        let old = self.set_current(new.clone());

        if old != new {
            old.cx.switch_to(new.cx)?;
        }

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

        loop {
            let steal = self.steal(worker);

            if steal.is_retry() {
                continue;
            }

            return steal.success();
        }
    }

    pub fn steal(&self, worker: &Worker<UThread>) -> Steal<UThread> {
        let steal = self
            .injector
            .steal_batch_with_limit_and_pop(worker, MAX_WORK_PER_WORKER);

        if !steal.is_empty() {
            return steal;
        }

        let mut stealers: Vec<_> = self.stealers.iter().collect();

        fastrand::shuffle(&mut stealers);

        stealers
            .iter()
            .map(|s| s.steal_batch_and_pop(worker))
            .find(|steal| steal.is_success())
            .unwrap_or(Steal::Empty)
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
