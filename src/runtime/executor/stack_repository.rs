use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{
    sys::stack::Stack,
    uthread::{Context, ReprContext},
    utils::IgnorePoison,
};

const TIMEOUT: Duration = Duration::from_secs(75);

#[derive(Default)]
pub struct StackRepository {
    inner: Mutex<Vec<Item>>,
}

pub struct Item {
    stack: Stack,
    time: Instant,
}

impl StackRepository {
    pub fn pop(&self, stack_size: usize) -> Option<Stack> {
        let mut stacks = self.inner.lock().ignore_poison();
        let i = stacks
            .iter()
            .position(|item| item.stack.size >= stack_size)?;
        Some(stacks.swap_remove(i).stack)
    }

    pub fn push(&self, cx: &Context) {
        debug_assert!(cx.has_exited());
        let ReprContext { stack, .. } = unsafe { &mut *cx.ptr() };
        let stack = std::mem::take(stack);
        self.inner.lock().ignore_poison().push(Item {
            stack,
            time: Instant::now(),
        });
    }

    pub fn clean_memory(&self) {
        self.inner
            .lock()
            .ignore_poison()
            .retain(|item| item.time.elapsed() < TIMEOUT)
    }
}
