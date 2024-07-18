use crossbeam_deque::{Stealer, Worker};
use thread_local::ThreadLocal;

use crate::uthread::{UThread, WorkerType};

use super::MAX_WORK_PER_WORKER;

#[derive(Default)]
pub(crate) struct Queue {
    pub worker: ThreadLocal<Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub injector: crossbeam_deque::Injector<UThread>,
}

impl Queue {
    pub fn push(&self, thread: UThread, ty: WorkerType) {
        if ty == WorkerType::BLOCKING_WORKER {
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
}
