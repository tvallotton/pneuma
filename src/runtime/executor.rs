use pneuma::thread::Thread;

use pneuma::thread::Stack;
use std::cell::UnsafeCell;
use std::io;

use pneuma::thread::Builder;
use pneuma::thread::JoinHandle;
use std::{cell::RefCell, collections::VecDeque};

use crate::sys;
use crate::thread::context::Status;

pub(crate) struct Executor {
    pub current: UnsafeCell<Thread>,
    pub run_queue: RefCell<VecDeque<Thread>>,
    pub all: RefCell<VecDeque<Thread>>,
    pub _unused_stacks: RefCell<Vec<Stack>>,
}

impl Executor {
    pub fn new() -> Executor {
        let thread = Thread::for_os_thread();
        let current = UnsafeCell::new(thread.clone());
        let all = RefCell::new(VecDeque::from([thread]));
        Executor {
            current,
            all,
            run_queue: RefCell::default(),
            _unused_stacks: RefCell::default(),
        }
    }

    pub fn current(&self) -> Thread {
        unsafe { &*self.current.get() }.clone()
    }

    /// Replaces the current thread with a new coroutine.
    #[inline]
    pub fn replace(&self, new: Thread) -> Thread {
        unsafe {
            let old = &*self.current.get();
            let old = old.clone();
            *self.current.get() = new;
            old
        }
    }
    #[inline]
    pub fn switch_to(&self, new: Thread) {
        let id = new.id();

        let old = self.replace(new.clone());

        if id != old.id() {
            new.status().set(Status::Waiting);
            unsafe { sys::switch_context(old.0 .0, new.0 .0) }
        }
    }

    pub fn spawn<T, F>(&self, f: F, builder: Builder) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let handle = JoinHandle::new(f, builder)?;
        self.all.borrow_mut().push_back(handle.thread().clone());
        self.push(handle.thread().clone());
        Ok(handle)
    }

    pub fn remove(&self, thread: &Thread) {
        let mut all = self.all.borrow_mut();

        let Some(i) = all.iter().position(|t| thread.eq(t)) else {
            return;
        };
        all.remove(i);
        dbg!(all.len());
    }

    pub fn total_threads(&self) -> usize {
        self.all.borrow().len()
    }

    pub fn unpark_all(&self) {
        for thread in &*self.all.borrow() {
            println!("{:?}", thread.id());
            thread.unpark()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.run_queue.borrow_mut().is_empty()
    }

    pub fn push(&self, thread: Thread) {
        self.run_queue.borrow_mut().push_back(thread);
    }

    pub fn pop(&self) -> Option<Thread> {
        self.run_queue.borrow_mut().pop_front()
    }
}
