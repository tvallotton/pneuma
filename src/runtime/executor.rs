use crossbeam_deque::{Steal, Stealer, Worker};
use pneuma::utils::IgnorePoison;
use queue::Queue;
use std::{
    mem::replace,
    sync::{
        atomic::{
            AtomicU64,
            Ordering::{Relaxed, Release},
        },
        Mutex,
    },
};
use thread_local::ThreadLocal;

use pneuma::{
    sys::stack::Stack,
    uthread::{Context, ReprContext, UThread},
};

use crate::uthread::QueueType;

use super::blocking_pool::BlockingPool;

const MAX_WORK_PER_WORKER: usize = 32;
mod queue;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub os_thread: ThreadLocal<UThread>,
    pub unused_stacks: Mutex<Vec<Stack>>,
    pub queue_type: ThreadLocal<QueueType>,
    pub blocking_pool: BlockingPool,
    pub async_queue: Queue,
    pub blocking_queue: Queue,
}

impl Executor {
    pub fn new() -> Executor {

        
        let mut exec = Executor::default();
        exec.blocking_queue.ty = QueueType::BLOCKING;
        exec
    }

    pub fn context_switch(&self) -> Result<(), ()> {
        let new = self.pop().ok_or(())?;

        new.cx.is_queued.store(false, Relaxed);

        let old = self.set_current(new.clone());

        if (old != new) && !new.cx.has_exited() && self.queue_type() == new.queue_type() {
            old.cx.switch_to(new.cx)?;
        }

        Ok(())
    }

    pub(crate) fn current_thread(&self) -> &Mutex<UThread> {
        self.current.get_or(|| {
            let os_thread = UThread::for_os_thread();
            self.os_thread.get_or(|| os_thread.clone());
            Mutex::new(os_thread)
        })
    }

    fn set_current(&self, with: UThread) -> UThread {
        let mut current = self.current_thread().lock().ignore_poison();
        replace(&mut *current, with)
    }

    fn pop(&self) -> Option<UThread> {
        let worker = self.queue().worker();

        let os_thread = self.pop_os_thread(worker);

        if os_thread.is_some() {
            return os_thread;
        }

        let thread = worker.pop();

        if thread.is_some() {
            return thread;
        }

        if self.work_stealing() {
            self.queue(self.queue_type()).steal(worker)
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
            .compare_exchange(queue_type, QueueType, Release, Relaxed)
            .ok()?;

        self.os_thread.get().cloned()
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
        let queue = self.queue(self.queue_type());
        let steal = queue
            .injector
            .steal_batch_with_limit_and_pop(worker, MAX_WORK_PER_WORKER);

        if !steal.is_empty() {
            return steal;
        }

        let mut stealers: Vec<_> = queue.stealers.iter().collect();

        fastrand::shuffle(&mut stealers);

        stealers
            .iter()
            .map(|s| s.steal_batch_and_pop(worker))
            .find(|steal| steal.is_success())
            .unwrap_or(Steal::Empty)
    }

    pub fn push(&self, thread: UThread) {
        self.queue(thread.queue_type()).push(thread)
    }

    pub(crate) fn recycle(&self, cx: &Context) {
        debug_assert!(cx.has_exited());
        let ReprContext { stack, .. } = unsafe { &mut *cx.ptr() };
        let stack = std::mem::take(stack);
        self.unused_stacks.lock().ignore_poison().push(stack);
    }

    pub(crate) fn stack(&self, stack_size: usize) -> Option<Stack> {
        let mut stacks = self.unused_stacks.lock().ignore_poison();
        let i = stacks.iter().position(|stack| stack.size >= stack_size)?;
        Some(stacks.swap_remove(i))
    }

    pub(crate) fn work_stealing(&self) -> bool {
        cfg!(feature = "unsafe_work_stealing") && self.queue_type() == QueueType::ASYNC
    }

    pub(crate) fn set_worker_type(&self, worker_type: QueueType) {
        self.queue_type.get_or(|| worker_type);
    }

    pub(crate) fn queue_type(&self) -> QueueType {
        *self.queue_type.get_or(|| QueueType::ASYNC)
    }

    pub(crate) fn queue(&self, queue_type: QueueType) -> &Queue {
        match queue_type {
            QueueType::ASYNC => &self.async_queue,
            QueueType::BLOCKING => &self.blocking_queue,
        }
    }
}
