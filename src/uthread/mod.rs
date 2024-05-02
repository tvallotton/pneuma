use context::Context;

use self::thread_id::UThreadId;
pub use join_handle::JoinHandle;

mod builder;
mod context;
mod join_handle;
mod lifecycle;
mod registers;
mod repr_context;
mod thread_id;

pub struct UThread {
    pub(crate) cx: Context,
}

impl UThread {
    fn new() -> UThread {
        todo!()
    }

    pub fn id(&self) -> UThreadId {
        self.cx.id
    }

    pub fn name(&self) -> Option<&str> {
        self.cx.name.as_deref()
    }
}

pub fn current() -> UThread {
    todo!()
}

pub fn park() -> std::io::Result<()> {
    todo!()
}
