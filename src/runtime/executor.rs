use crossbeam_deque::Steal;
use global_queue::GlobalQueue;
use local_queue::LocalQueue;
use pneuma::utils::IgnorePoison;
use std::{
    alloc::GlobalAlloc,
    mem::replace,
    ptr,
    sync::{
        atomic::Ordering::{Relaxed, Release},
        Mutex,
    },
};
use thread_local::ThreadLocal;
use worker::{Stealer, Worker};

use pneuma::{
    sys::stack::Stack,
    uthread::{Context, ReprContext, UThread},
};

mod global_queue;
mod local_queue;
#[cfg(test)]
mod test;
mod worker;

const MAX_WORK_PER_WORKER: usize = 16;

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub os_thread: ThreadLocal<UThread>,
    pub worker: ThreadLocal<Worker<UThread>>,
    pub stealers: ThreadLocal<Stealer<UThread>>,
    pub global: GlobalQueue<UThread>,
    // pub global: GlobalQueue<UThread>,
    pub unused_stacks: Mutex<Vec<Stack>>,
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
        };

        let thread = worker.pop();

        if thread.is_some() {
            return thread;
        }

        if cfg!(feature = "unsafe_work_stealing") {
            dbg!();
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
        dbg!();
        self.try_steal(worker);

        dbg!(self.worker().pop())
    }

    pub fn try_steal(&self, worker: &Worker<UThread>) {
        let stolen = self
            .worker()
            .push_batch(|| self.global.pop_batch().into_iter());

        if stolen > 0 {
            return;
        }
        dbg!();
        let mut stealers: Vec<_> = self.stealers.iter().collect();
        dbg!();
        fastrand::shuffle(&mut stealers);

        dbg!(&stealers);

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
            return self.global.push(thread);
        }

        let Some(thread) = self.worker().push(thread) else {
            return;
        };

        self.global.push(thread);
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

    // pub(crate) fn mark_as_blocking(&self) {
    //     let worker = self.worker();

    //     while let Some(thread) = worker.pop() {
    //         self.global.push_batch(nodes);
    //     }

    //     while let Some(work) = worker.pop() {
    //         self.global.push(work);
    //     }
    // }
}
