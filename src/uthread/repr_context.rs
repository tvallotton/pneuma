use std::{
    alloc::Layout,
    any::Any,
    cell::UnsafeCell,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8},
        Mutex,
    },
    thread::Thread,
};

use super::{registers::Registers, thread_id::UThreadId, UThread};
use crate::sys::stack::Stack;

pub struct ReprContext {
    pub registers: UnsafeCell<Registers>,

    pub stack: Stack,

    pub lifecycle: AtomicU8,

    pub is_queued: AtomicBool,

    pub is_running: AtomicBool,

    pub refcount: AtomicU64,

    pub join_waker: Mutex<Option<UThread>>,

    pub fun: *mut dyn FnMut(*mut ()),

    pub out: *mut dyn Any,
    /// immutable
    pub name: Option<String>,
    /// immutable
    pub layout: Layout,
    /// immutable
    pub id: UThreadId,
}
