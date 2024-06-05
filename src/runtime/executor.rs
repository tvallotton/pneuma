use crossbeam_deque::{Steal, Stealer, Worker};
use pneuma::utils::IgnorePoison;
use std::{
    mem::replace,
    sync::{
        atomic::Ordering::{Relaxed, Release},
        Mutex,
    },
};
use thread_local::ThreadLocal;

use crate::{
    sys::stack::Stack,
    uthread::{Context, ReprContext, UThread},
};

const MAX_WORK_PER_WORKER: usize = 16;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub os_thread: ThreadLocal<UThread>,
    pub worker: ThreadLocal<crossbeam_deque::Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub injector: crossbeam_deque::Injector<UThread>,
    pub unused_stacks: Mutex<Vec<Stack>>,
}

impl Executor {
    pub fn context_switch(&self) -> Result<(), ()> {
        let new = self.pop().ok_or(())?;

        new.cx.is_queued.store(false, Relaxed);

        let old = self.set_current(new.clone());

        if (old != new) && !new.cx.has_exited() {
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
        let worker = self.worker();

        let os_thread = self.pop_os_thread(worker);

        if os_thread.is_some() {
            return os_thread;
        }

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

    pub fn worker(&self) -> &Worker<UThread> {
        self.worker.get_or(|| {
            let worker = crossbeam_deque::Worker::new_fifo();
            self.stealers.get_or(|| worker.stealer());
            worker
        })
    }

    pub fn push(&self, thread: UThread) {
        if fastrand::u8(0..12) == 0 {
            return self.injector.push(thread);
        }

        let worker = self.worker();

        if worker.len() < MAX_WORK_PER_WORKER {
            return worker.push(thread);
        }

        self.injector.push(thread)
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
}
