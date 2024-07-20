use super::{park, yield_now, QueueType};
use std::sync::atomic::Ordering::Relaxed;

pub fn block<F, T>(f: F)
where
    F: FnOnce() -> T,
{
    let rt = pneuma::runtime();

    let uthread = pneuma::uthread::current();
    uthread.set_queue_type(QueueType::BLOCKING);

    uthread.cx.is_queued.store(true, Relaxed);
    rt.executor.blocking_queue.injector.push(uthread);

    while let QueueType::ASYNC = rt.executor.queue_type() {
        yield_now();
    }

    let output = f();
}
