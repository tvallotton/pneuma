use libc::sched_yield;
use std::array;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::yield_now;

use crate::runtime::executor::worker::Stealer;

use super::local_queue;
use super::GlobalQueue;
use super::LocalQueue;
use super::Worker;

#[test]
fn push_and_pop() {
    let local_queue = LocalQueue::with_capacity(8);

    local_queue.push(9);
    assert_eq!(local_queue.pop(), Some(9));
}

#[test]
fn push_and_pop_batch() {
    let local_queue = LocalQueue::with_capacity(8);

    local_queue.push_batch(|| [1, 2, 3, 4].into_iter());
    let mut batch = local_queue.pop_batch().into_iter();
    let array: [_; 2] = array::from_fn(|_| batch.next().unwrap());
    assert_eq!(array, [1, 2]);
}

#[test]
fn concurrent_push_and_pop() {
    const NUMBER_OF_THREADS: usize = 10;
    let worker = Worker::with_capacity(16);

    let ref total = AtomicUsize::new(0);

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            let stealer = worker.stealer();

            s.spawn(move || loop {
                let Some(loot) = stealer.pop() else {
                    break;
                };
                total.fetch_add(loot, Relaxed);

                std::thread::yield_now();
            });
        }

        for i in 0..100 {
            loop {
                if let None = worker.push(i) {
                    break;
                }
                std::thread::yield_now();
            }
        }
    });
    assert_eq!(total.load(Relaxed), 4950)
}

#[test]
fn batched_concurrent_push_and_pop() {
    const NUMBER_OF_THREADS: usize = 10;
    let worker = Worker::with_capacity(16);

    let ref total = AtomicUsize::new(0);
    let ref stop = AtomicBool::new(false);

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            let stealer = worker.stealer();

            s.spawn(move || {
                let mut loots = vec![];
                loop {
                    let loot = stealer.pop_batch().into_iter().sum();
                    loots.push(loot);
                    total.fetch_add(loot, Relaxed);

                    std::thread::yield_now();

                    if loot == 0 && stop.load(Relaxed) {
                        break;
                    }
                }
            });
        }
        let mut values = (0..100).into_iter().peekable();

        loop {
            if dbg!(values.peek()).is_none() {
                stop.store(true, Relaxed);
                break;
            }
            dbg!(worker.push_batch(|| &mut values));

            std::thread::yield_now();
        }
    });
    assert_eq!(total.load(Relaxed), 4950)
}
