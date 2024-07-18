pub use WorkerType::*;

// LIFECYCLE

pub const NEW: u8 = 0;
pub const RUNNING: u8 = 1;
pub const FINISHED: u8 = 2;
pub const TAKEN: u8 = 3;
pub const OS_THREAD: u8 = 4;

// WORKER QUEUES

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WorkerType {
    ASYNC_WORKER = 0,
    BLOCKING_WORKER = 1,
}

pub const UNLOCKED: bool = false;
pub const LOCKED: bool = true;
