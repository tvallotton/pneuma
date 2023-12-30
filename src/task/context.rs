use pneuma::task::RcContext;

use super::{registers::Registers, stack::Stack};
use std::alloc::alloc;
use std::alloc::Layout;
use std::any::Any;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::io;
use std::mem::zeroed;
use std::ptr::NonNull;

/// The thread context as it was left before the switch.
///
/// # Allocation
/// It is import to remember that the context allocation
/// is extended to contain the closure and its output.

#[repr(C)]
pub(crate) struct Context {
    pub registers: UnsafeCell<Registers>,
    pub stack: Stack,
    pub layout: Layout,
    pub status: Cell<Status>,
    pub refcount: Cell<u64>,
    pub fun: *mut dyn FnMut(*mut ()),
    pub out: *mut dyn Any,
    // fun_alloc: impl FnMut(&mut Option<T>),
    // out_alloc: Result<T, Box<dyn Any + Send + 'static>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(C)]
pub enum Status {
    New = 0,
    Running = 1,
    Finished = 2,
    Taken = 3,
    OsThread = 4,
}

impl Context {
    pub fn new<T, F>(size: usize, fun: F) -> io::Result<RcContext>
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        unsafe {
            let (layout, fun_offset, out_offset) = layout::<T, F>();

            let ptr = alloc(layout);
            assert!(!ptr.is_null());

            let fun_alloc = ptr.add(fun_offset) as *mut F;
            fun_alloc.write(fun);

            let out = ptr
                .add(out_offset)
                .cast::<Result<T, Box<dyn Any + Send + 'static>>>()
                as *mut dyn Any;

            let cx = Context {
                registers: zeroed(),
                stack: Stack::new(size)?,
                refcount: 1.into(),
                fun: fun_alloc as *mut dyn FnMut(*mut ()),
                status: Status::New.into(),
                layout,
                out,
            };
            ptr.cast::<Context>().write(cx);
            let cx = RcContext(NonNull::new(ptr.cast()).unwrap());
            Ok(cx.setup_registers())
        }
    }

    pub fn for_os_thread() -> RcContext {
        let mut cx = Self::new::<(), _>(0, |_| ()).unwrap();
        cx.status.set(Status::OsThread);
        cx
    }
}

pub(crate) fn layout<T, F>() -> (Layout, usize, usize) {
    let raw_task = Layout::new::<Context>();
    let fun = Layout::new::<F>();
    let out = Layout::new::<Result<T, Box<dyn Any + Send + 'static>>>();
    let (layout, fun) = raw_task.extend(fun).unwrap();
    let (layout, out) = layout.extend(out).unwrap();
    (layout, fun, out)
}
