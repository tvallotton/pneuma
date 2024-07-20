use crossbeam_deque::{Steal, Stealer, Worker};
use thread_local::ThreadLocal;

use crate::uthread::{QueueType, UThread};

use super::MAX_WORK_PER_WORKER;

#[derive(Default)]
pub(crate) struct Queue {
    pub ty: QueueType,
    pub worker: ThreadLocal<Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub injector: crossbeam_deque::Injector<UThread>,
}

impl Queue {
    pub fn push(&self, thread: UThread) {
        if self.ty == QueueType::BLOCKING {
            return self.injector.push(thread);
        }

        if fastrand::u8(0..12) == 0 {
            return self.injector.push(thread);
        }

        let worker = self.worker();

        if worker.len() < MAX_WORK_PER_WORKER {
            return worker.push(thread);
        }

        self.injector.push(thread)
    }

    fn pop(&self) -> Option<UThread> {
        let worker = self.worker();

        let os_thread = self.pop_os_thread(worker);

        if os_thread.is_some() {
            return os_thread;
        }

        let thread = worker.pop();

        if thread.is_some() {
            return thread;
        }

        if self.work_stealing() {
            return self.steal(worker);
        }

        None
    }

    pub fn worker(&self) -> &Worker<UThread> {
        self.worker.get_or(|| {
            let worker = Worker::new_fifo();
            self.stealers.get_or(|| worker.stealer());
            worker
        })
    }

    pub fn steal(&self, worker: &Worker<UThread>) -> Option<UThread> {
        loop {
            let steal = self.try_steal(worker);

            if steal.is_retry() {
                continue;
            }
            return steal.success();
        }
    }
    pub fn try_steal(&self, worker: &Worker<UThread>) -> Steal<UThread> {
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
}
