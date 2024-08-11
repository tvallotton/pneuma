//! A SPMC Queue
use std::array;
use std::mem::forget;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering::*};
use std::sync::Arc;
use std::{mem::MaybeUninit, sync::atomic::AtomicUsize};

use super::MAX_WORK_PER_WORKER;

const MAX_NUM_RETRIES: usize = 5;
pub const IS_EMPTY: bool = false;
pub const TRY_AGAIN: bool = true;

/// A single produce multi consumer queue.
pub struct LocalQueue<T> {
    // updated by multiple consumers
    head: AtomicUsize,
    // only updated by the worker thread
    tail: AtomicUsize,
    capacity: usize,
    buffer: *mut MaybeUninit<T>,
    mask: usize,
}

impl<T> LocalQueue<T> {
    pub fn with_capacity(capacity: usize) -> LocalQueue<T> {
        let capacity = capacity.next_power_of_two();
        let mut alloc = Vec::with_capacity(capacity);
        let buffer = alloc.as_mut_ptr();
        forget(alloc);
        LocalQueue {
            head: 0.into(),
            tail: 0.into(),
            capacity,
            buffer,
            mask: (1 << capacity.ilog2() as usize) - 1,
        }
    }

    pub fn pop(&self) -> Option<T> {
        loop {
            return match unsafe { self.try_pop() } {
                Ok(val) => Some(val),
                Err(TRY_AGAIN) => continue,
                Err(IS_EMPTY) => None,
            };
        }
    }

    pub fn pop_batch(&self) -> impl DoubleEndedIterator<Item = T> {
        let mut buffer: [MaybeUninit<T>; MAX_WORK_PER_WORKER] =
            array::from_fn(|_| MaybeUninit::uninit());

        let output = |buffer: [MaybeUninit<T>; MAX_WORK_PER_WORKER]| {
            buffer.into_iter().map(|mu| unsafe { mu.assume_init() })
        };

        for _ in 0..MAX_NUM_RETRIES {
            let Some(read) = (unsafe { self.try_pop_batch(&mut buffer) }) else {
                continue;
            };

            return output(buffer).take(read);
        }

        output(buffer).take(0)
    }

    #[inline]
    unsafe fn try_pop(&self) -> Result<T, bool> {
        let head = self.head.load(Acquire);
        let tail = self.tail.load(Relaxed);

        let is_empty = head == tail;

        if is_empty {
            return Err(IS_EMPTY);
        }

        let slot = head & self.mask;

        let item = self.buffer.add(slot).read();

        self.head
            .compare_exchange(head, head.wrapping_add(1), Release, Relaxed)
            .map_err(|_| TRY_AGAIN)?;

        Ok(item.assume_init())
    }

    #[inline]
    unsafe fn try_pop_batch(&self, buffer: &mut [MaybeUninit<T>]) -> Option<usize> {
        let mut head = self.head.load(Acquire);
        let tail = self.tail.load(Relaxed);
        let to_steal = (1 + tail.saturating_sub(head)) / 2;
        let old_head = head;

        for i in 0..to_steal {
            let Some(out) = buffer.get_mut(i) else {
                break;
            };
            let slot = head & self.mask;
            *out = self.buffer.add(slot).read();
            head += 1;
        }

        self.head
            .compare_exchange(old_head, head, Release, Relaxed)
            .ok()?;

        Some(to_steal)
    }

    pub fn push(&self, item: T) -> Option<T> {
        let head = self.head.load(Acquire);
        let tail = self.tail.load(Relaxed);
        let slot = tail & self.mask;

        let is_full = tail.wrapping_sub(head) == self.capacity;

        if is_full {
            return Some(item);
        }

        unsafe { self.buffer.add(slot).write(MaybeUninit::new(item)) };

        self.tail.store(tail.wrapping_add(1), Release);
        return None;
    }

    pub fn push_batch<F, I>(&self, batch: F) -> usize
    where
        F: FnOnce() -> I,
        I: Iterator<Item = T>,
    {
        let head = self.head.load(Acquire);
        let mut tail = self.tail.load(Relaxed);

        let filled = tail.wrapping_sub(head);
        let available = self.capacity.saturating_sub(filled);

        let old_tail = tail;

        for item in batch().take(available) {
            let slot = tail & self.mask;

            unsafe { self.buffer.add(slot).write(MaybeUninit::new(item)) }
            tail = tail.wrapping_add(1);
        }

        self.tail.store(tail, Release);
        return tail - old_tail;
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Acquire) == self.tail.load(Relaxed)
    }
    // Not a perfect len, it could be lower than the actual length
    pub fn len(&self) -> usize {
        self.tail
            .load(Relaxed)
            .saturating_sub(self.head.load(Acquire))
    }
}

impl<T> Drop for LocalQueue<T> {
    fn drop(&mut self) {
        let head = *self.head.get_mut();
        let tail = *self.tail.get_mut();

        for i in 0..(tail.wrapping_sub(head)) {
            unsafe { self.buffer.add(i).drop_in_place() };
        }
    }
}
