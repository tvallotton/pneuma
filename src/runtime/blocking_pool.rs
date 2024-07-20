use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    str::Utf8Chunk,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use crate::{
    uthread::{self, UThread, QueueType},
    utils::IgnorePoison,
};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::sync::atomic::Ordering::Relaxed;

const MAX_WORKERS: usize = 512;
const KEEP_ALIVE: Duration = Duration::from_secs(15);

pub struct BlockingPool {
    pub n_workers: AtomicUsize,

}

impl BlockingPool {
    fn new() -> Self {
        let (tx_rendezvous, rx_rendezvous) = bounded(0);
        let (tx_unbounded, rx_unbounded) = unbounded();
        BlockingPool {
            n_workers: 0.into(),
            tx_rendezvous,
            rx_rendezvous,
            tx_unbounded,
            rx_unbounded,
        }
    }

    pub fn block<T, F>(self: Arc<Self>, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.clone().switch_to_blocking_pool();
        let result = catch_unwind(AssertUnwindSafe(f));
        self.switch_from_blocking_pool();
        match result {
            Ok(out) => out,
            Err(err) => resume_unwind(err),
        }
    }

    fn switch_to_blocking_pool(self: Arc<Self>) {
        let uthread = pneuma::uthread::current();
        self.send(uthread);
        pneuma::runtime().park(QueueType::BLOCKING);
    }

    fn switch_from_blocking_pool(self: Arc<Self>) {
        let uthread = pneuma::uthread::current();
        uthread.unpark();
        pneuma::runtime().park(QueueType::ASYNC);
        todo!()
    }

    fn send(self: Arc<Self>, uthread: UThread) {
        let Err(result) = self.tx_rendezvous.try_send(uthread) else {
            return;
        };

        let uthread = result.into_inner();

        self.clone().try_spawn();

        self.tx_unbounded.send(uthread);
    }

    fn try_spawn(self: Arc<Self>) {
        let n_workers = self.n_workers.fetch_add(1, Relaxed);

        if n_workers + 1 >= MAX_WORKERS {
            self.n_workers.fetch_sub(1, Relaxed);
            return;
        }
        self.spawn(n_workers);
    }

    fn spawn(self: Arc<Self>, index: usize) -> Result<(), std::io::Error> {
        std::thread::Builder::new()
            .name(format!("pneuma_blocking_pool_worker[{index}]"))
            .spawn(move || self.worker_loop())?;
        Ok(())
    }

    fn worker_loop(self: Arc<Self>, option: Option<UThread>) {
        let executor = pneuma::runtime().executor;
        executor.set_worker_type(QueueType::BLOCKING);

        let rx = [&self.rx_rendezvous, &self.rx_unbounded];
        let mut select = crossbeam_channel::Select::new();
        select.recv(rx[0]);
        select.recv(rx[1]);

        while let Ok(op) = select.select_timeout(KEEP_ALIVE) {
            let i = op.index();

            let Ok(uthread) = op.recv(rx[i]) else {
                continue;
            };

            uthread.unpark()
            executor.context_switch();
        }
    }
}

impl Default for BlockingPool {
    fn default() -> Self {
        Self::new()
    }
}
