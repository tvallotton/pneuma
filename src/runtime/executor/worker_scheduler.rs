use std::num::NonZero;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;
use std::time::Duration;

use crate::uthread::{self, park, UThread};
use crate::utils::IgnorePoison;

pub struct WorkerScheduler(Mutex<Inner>);

struct Inner {
    total_n_workers: u16,
    n_blocking_workers: u16,
    min_n_workers: u16,
    max_n_workers: u16,
    back_pressure_queue: Vec<UThread>,
}

impl Default for WorkerScheduler {
    fn default() -> Self {
        WorkerScheduler(Mutex::new(Inner {
            total_n_workers: 0,
            n_blocking_workers: 0,
            max_n_workers: 256,
            back_pressure_queue: vec![],
            min_n_workers: std::thread::available_parallelism()
                .map(|i| i.get())
                .unwrap_or(1) as u16,
        }))
    }
}

impl WorkerScheduler {
    pub fn set_bounds(&self, max: u16, min: u16) {
        assert!(
            0 < max,
            "The maximum number of workers allowed must be greater than zero."
        );
        assert!(
            min < max,
            "The maximum number of workers allowed must be greater than the minimum."
        );

        let mut count = self.0.lock().ignore_poison();
        count.max_n_workers = max;
        count.min_n_workers = min;
    }

    pub fn mark_as_blocking(&self) {
        self.0.lock().ignore_poison().mark_as_blocking();
    }

    pub fn mark_as_async(&self) -> Option<()> {
        let mut sched = self.0.lock().ignore_poison();
        sched.n_blocking_workers -= 1;
        sched.back_pressure_queue.pop()?.unpark();
        Some(())
    }
}

impl Inner {
    pub fn mark_as_blocking(&mut self) {
        self.n_blocking_workers += 1;

        if !self.needs_more_workers() {
            return;
        }

        if self.can_spawn_more_workers() {
            self.spawn_worker();
        } else {
            self.wait_for_a_blocking_worker_to_become_async();
        }
    }

    fn needs_more_workers(&self) -> bool {
        return self.total_n_workers - self.n_blocking_workers < self.min_n_workers;
    }

    fn can_spawn_more_workers(&self) -> bool {
        return self.total_n_workers < self.max_n_workers;
    }

    fn wait_for_a_blocking_worker_to_become_async(&mut self) {
        let uthread = uthread::current();
        self.back_pressure_queue.push(uthread);
        park();
    }

    // TODO: find a smarter way of exiting from the worker
    fn spawn_worker(&mut self) {
        self.total_n_workers += 1;
        std::thread::spawn(|| loop {
            pneuma::time::sleep(Duration::from_secs(30));
            let elapsed = pneuma::runtime()
                .executor
                .current
                .get()
                .unwrap()
                .lock()
                .ignore_poison()
                .1
                .elapsed();

            if Duration::from_secs(30) < elapsed {
                pneuma::runtime().executor.block_worker();
                return;
            }
        });
    }
}
