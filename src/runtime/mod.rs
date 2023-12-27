use std::rc::Rc;

// use crate::reactor::Reactor;
// use crate::thread::JoinHandle;
use executor::Executor;

mod executor;
mod globals;

#[derive(Clone)]
pub struct Runtime(Rc<InnerRuntime>);

pub struct InnerRuntime {
    executor: Executor,
    // reactor: Reactor,
}

impl Runtime {
    // fn spawn<'a, F, T>(&self, f: F) -> JoinHandle<'a, T>
    // where
    //     F: FnOnce() -> T + 'a,
    // {
    //     self.executor.spawn(f)
    // }

    fn block_on<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        todo!()
    }
}
