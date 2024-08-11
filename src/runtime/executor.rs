use crossbeam_deque::Steal;
use global_queue::GlobalQueue;
use local_queue::LocalQueue;
use pneuma::utils::IgnorePoison;
use stack_repository::StackRepository;
use std::{
    alloc::GlobalAlloc,
    mem::replace,
    ptr,
    sync::{
        atomic::{
            AtomicU32,
            Ordering::{Relaxed, Release},
        },
        Mutex,
    },
    time::{Duration, Instant},
};
use thread_local::ThreadLocal;
use worker::{Stealer, Worker};
use worker_scheduler::WorkerScheduler;

use pneuma::{
    sys::stack::Stack,
    uthread::{Context, ReprContext, UThread},
};

mod global_queue;
mod local_queue;
mod stack_repository;
#[cfg(test)]
mod test;
mod worker;
mod worker_scheduler;

const MAX_WORK_PER_WORKER: usize = 16;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<(UThread, Instant)>>,
    pub os_thread: ThreadLocal<UThread>,
    pub worker: ThreadLocal<Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub global: GlobalQueue<UThread>,
    pub worker_scheduler: WorkerScheduler,
    pub stack_repository: StackRepository,
}

impl Executor {
    pub fn context_switch(&self) -> Result<(), ()> {
        let new = self.pop().ok_or(())?;
        new.cx.is_queued.store(false, Relaxed);
        let old = self.set_current(new.clone());
        if (old != new) && !new.cx.has_exited() {
            old.cx.switch_to(new.cx)?;
        };
        Ok(())
    }

    pub(crate) fn current_thread(&self) -> &Mutex<(UThread, Instant)> {
        self.current.get_or(|| {
            let instant = Instant::now();
            let os_thread = UThread::for_os_thread();
            self.os_thread.get_or(|| os_thread.clone());
            Mutex::new((os_thread, instant))
        })
    }

    fn set_current(&self, with: UThread) -> UThread {
        let instant = Instant::now();
        let mut current = self.current_thread().lock().ignore_poison();
        replace(&mut *current, (with, instant)).0
    }

    fn pop(&self) -> Option<UThread> {
        let worker = self.worker();
        let os_thread = self.pop_os_thread(worker);
        if os_thread.is_some() {
            return os_thread;
        };

        let thread = worker.pop();

        if thread.is_some() {
            return thread;
        }

        if cfg!(feature = "unsafe_work_stealing") && !worker.is_blocked.load(Relaxed) {
            return self.steal(worker);
        }

        None
    }

    fn pop_os_thread(&self, worker: &Worker<UThread>) -> Option<UThread> {
        if fastrand::usize(0..(worker.len() + 1)) != 0 {
            return None;
        };

        let os_thread = self.os_thread.get().cloned()?;

        os_thread
            .cx
            .is_queued
            .compare_exchange(true, false, Release, Relaxed)
            .ok()?;

        self.os_thread.get().cloned()
    }

    pub fn steal(&self, worker: &Worker<UThread>) -> Option<UThread> {
        self.try_steal(worker);
        self.worker().pop()
    }

    pub fn try_steal(&self, worker: &Worker<UThread>) {
        let stolen = self
            .worker()
            .push_batch(|| self.global.pop_batch_front().into_iter());

        if stolen > 0 {
            return;
        }

        let mut stealers: Vec<_> = self.stealers.iter().collect();

        fastrand::shuffle(&mut stealers);

        stealers
            .iter()
            .map(|s| worker.push_batch(|| s.pop().into_iter()))
            .filter(|x| *x > 1)
            .take(1)
            .for_each(|stolen| {
                dbg!(stolen);
            });
    }

    pub fn worker(&self) -> &Worker<UThread> {
        self.worker.get_or(|| {
            let worker = Worker::with_capacity(MAX_WORK_PER_WORKER);
            self.stealers.get_or(|| worker.stealer());
            worker
        })
    }

    pub fn push(&self, thread: UThread) {
        if fastrand::u8(0..12) == 0 {
            return self.global.push_back(thread);
        }

        let worker = self.worker();

        if worker.is_blocked.load(Relaxed) {
            return self.global.push_back(thread);
        }

        let Some(thread) = worker.push(thread) else {
            return;
        };

        self.global.push_back(thread);
    }

    pub(crate) fn recycle(&self, cx: &Context) {
        self.stack_repository.push(cx)
    }

    pub(crate) fn stack(&self, stack_size: usize) -> Option<Stack> {
        self.stack_repository.pop(stack_size)
    }

    pub(crate) fn block_worker(&self) {
        let worker = self.worker();
        let batch = worker.pop_batch();
        self.global.push_batch_front(batch.into_iter());
        worker.is_blocked.store(false, Relaxed);
    }

    pub(crate) fn unblock_worker(&self) {
        self.worker().is_blocked.store(true, Relaxed)
    }

    pub(crate) fn free_unused_memory(&self) {
        self.stack_repository.clean_memory();
    }
}
