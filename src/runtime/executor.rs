use crossbeam_deque::{Injector, Steal};
use pneuma::utils::IgnorePoison;
use stack_repository::StackRepository;
use std::{
    mem::replace,
    sync::{
        atomic::Ordering::{Relaxed, Release},
        Mutex,
    },
    time::Instant,
};
use thread_local::ThreadLocal;

use crossbeam_deque::{Stealer, Worker};
use pneuma::{
    sys::stack::Stack,
    uthread::{Context, UThread},
};

mod global_queue;
mod local_queue;
mod stack_repository;

mod worker;
mod worker_scheduler;

const MAX_WORK_PER_WORKER: usize = 16;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<(UThread, Instant)>>,
    pub os_thread: ThreadLocal<UThread>,
    pub worker: ThreadLocal<Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub global: Injector<UThread>,
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

        if cfg!(feature = "unsafe_work_stealing") {
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

    // TODO: use steal_batch_and_pop
    pub fn try_steal(&self, worker: &Worker<UThread>) {
        let steal = self.global.steal_batch(worker);

        if steal.is_success() {
            return;
        }

        let mut stealers: Vec<_> = self.stealers.iter().collect();

        fastrand::shuffle(&mut stealers);

        stealers
            .iter()
            .filter(|s| s.steal_batch(worker).is_success())
            .take(1)
            .for_each(|_| {});
    }

    pub fn worker(&self) -> &Worker<UThread> {
        self.worker.get_or(|| {
            let worker = Worker::new_fifo();
            self.stealers.get_or(|| worker.stealer());
            worker
        })
    }

    pub fn push(&self, thread: UThread) {
        let worker = self.worker();

        if MAX_WORK_PER_WORKER <= worker.len() {
            return self.global.push(thread);
        }

        worker.push(thread);
    }

    pub(crate) fn recycle(&self, cx: &Context) {
        self.stack_repository.push(cx)
    }

    pub(crate) fn stack(&self, stack_size: usize) -> Option<Stack> {
        self.stack_repository.pop(stack_size)
    }

    pub(crate) fn free_unused_memory(&self) {
        self.stack_repository.clean_memory();
    }
}
