use std::{
    fmt::Debug,
    sync::{atomic::AtomicBool, Arc},
};

use crate::uthread::UThread;

use super::local_queue::LocalQueue;

pub struct Worker<T> {
    pub is_blocked: AtomicBool,
    queue: Arc<LocalQueue<T>>,
}

pub struct Stealer<T> {
    queue: Arc<LocalQueue<T>>,
}

impl<T> Worker<T> {
    pub fn with_capacity(capacity: usize) -> Worker<T> {
        let queue = LocalQueue::with_capacity(capacity);
        let queue = Arc::new(queue);
        let is_blocked = false.into();
        Worker { queue, is_blocked }
    }

    pub fn push(&self, item: T) -> Option<T> {
        self.queue.push(item)
    }

    pub fn push_batch<F, I>(&self, batch: F) -> usize
    where
        F: FnOnce() -> I,
        I: Iterator<Item = T>,
    {
        self.queue.push_batch(batch)
    }

    pub fn stealer(&self) -> Stealer<T> {
        Stealer {
            queue: self.queue.clone(),
        }
    }

    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    pub fn pop_batch(&self) -> impl DoubleEndedIterator<Item = T> {
        self.queue.pop_batch()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl<T> Stealer<T> {
    pub fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    pub fn pop_batch(&self) -> impl IntoIterator<Item = T> {
        self.queue.pop_batch()
    }
}

unsafe impl<T: Send> Send for Worker<T> {}
unsafe impl<T: Send> Send for Stealer<T> {}
unsafe impl<T: Sync> Sync for Stealer<T> {}

impl<T> Debug for Stealer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stealer")
            .field("len", &self.queue.len())
            .finish()
    }
}
