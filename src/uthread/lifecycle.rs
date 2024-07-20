use std::default;

pub use QueueType::*;

// LIFECYCLE

pub const NEW: u8 = 0;
pub const RUNNING: u8 = 1;
pub const FINISHED: u8 = 2;
pub const TAKEN: u8 = 3;
pub const OS_THREAD: u8 = 4;

// WORKER QUEUES

pub const UNLOCKED: bool = false;
pub const LOCKED: bool = true;

#[repr(u8)]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    #[default]
    ASYNC = 0,
    BLOCKING = 1,
}

impl From<u8> for QueueType {
    fn from(value: u8) -> Self {
        match value & 1 {
            1 => BLOCKING,
            _ => ASYNC,
        }
    }
}

impl Into<u8> for QueueType {
    fn into(self) -> u8 {
        self as u8
    }
}
