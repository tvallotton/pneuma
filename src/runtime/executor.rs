use std::{cell::RefCell, collections::VecDeque, sync::Mutex};

use thread_local::ThreadLocal;

use crate::{sys::stack::Stack, uthread::UThread};

#[derive(Default)]
pub(crate) struct Executor {
    pub current: ThreadLocal<Mutex<UThread>>,
    pub local_queue: ThreadLocal<Mutex<VecDeque<UThread>>>,
    pub global_queue: Mutex<VecDeque<UThread>>,
    pub all: Mutex<VecDeque<UThread>>,
    pub _unused_stacks: Mutex<Vec<Stack>>,
}

impl Executor {}
