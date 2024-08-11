//! A SPMC Queue
use std::mem::{forget, transmute};
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{
    AtomicI32, AtomicPtr, AtomicU32,
    Ordering::{self, *},
};
use std::sync::Mutex;
use std::{mem::MaybeUninit, sync::atomic::AtomicUsize};

use crate::uthread::{Context, ReprContext, UThread};
use crate::utils::IgnorePoison;

pub struct GlobalQueue<T: Send + Sync + Node> {
    queue: Mutex<Inner<T>>,
}

pub struct Inner<T: Node> {
    head: *mut T::Repr,
    tail: *mut T::Repr,
}

trait Node {
    type Repr;
    fn into_repr(self) -> *mut Self::Repr;
    fn from_repr(repr: *mut Self::Repr) -> Self;
    fn next(&self) -> &mut *mut Self::Repr;
}

impl<T: Send + Sync + Node> GlobalQueue<T> {
    fn new() -> Self {
        GlobalQueue {
            queue: Mutex::new(Inner {
                head: null_mut(),
                tail: null_mut(),
            }),
        }
    }

    pub fn push(&self, node: T) {
        self.push_batch([node])
    }

    pub fn pop(&self) -> Option<T> {
        self.pop_batch().into_iter().next()
    }

    pub fn push_batch(&self, nodes: impl IntoIterator<Item = T>) {
        let mut queue = self.queue.lock().ignore_poison();
        for node in nodes {
            *node.next() = null_mut();

            if queue.head.is_null() {
                queue.tail = node.into_repr();
                queue.head = queue.tail;

                continue;
            }

            let tail: T = Node::from_repr(queue.tail);

            *tail.next() = node.into_repr();
            queue.tail = *tail.next();

            forget(tail);
        }
    }

    pub fn pop_batch(&self) -> impl IntoIterator<Item = T> + '_ {
        let mut queue = self.queue.lock().ignore_poison();
        std::iter::from_fn(move || {
            if queue.head.is_null() {
                return None;
            }
            let head: T = Node::from_repr(queue.head);

            queue.head = *head.next();

            Some(head)
        })
    }
}

impl Node for UThread {
    type Repr = ReprContext;
    #[track_caller]
    fn from_repr(repr: *mut Self::Repr) -> Self {
        println!("{}", std::panic::Location::caller());
        UThread {
            cx: Context {
                ptr: { NonNull::new(repr).unwrap() },
            },
        }
    }

    fn into_repr(self) -> *mut Self::Repr {
        unsafe { transmute(self) }
    }

    fn next(&self) -> &mut *mut Self::Repr {
        unsafe { &mut (&mut *self.cx.ptr.as_ptr()).next }
    }
}
impl<T: Node + Send + Sync> Default for GlobalQueue<T> {
    fn default() -> Self {
        GlobalQueue::new()
    }
}

unsafe impl<T: Sync + Node> Sync for Inner<T> {}
unsafe impl<T: Send + Node> Send for Inner<T> {}
