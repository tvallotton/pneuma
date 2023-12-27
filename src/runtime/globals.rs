use super::Runtime;
use std::{cell::Cell, ptr::null, rc::Rc};

thread_local! {
    static RUNTIME: Cell<*const Runtime> = const { Cell::new(null()) };
}

fn runtime() -> Option<Runtime> {
    let runtime = RUNTIME.get();
    if runtime.is_null() {
        return None;
    }
    unsafe { Some((&*runtime).clone()) }
}
