use pneuma::thread::RcContext;

use crate::runtime::Runtime;

use super::builder::Builder;
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
    pub runtime: Runtime,
    pub name: Option<String>,
    pub lifecycle: Cell<Lifecycle>,
    pub status: Cell<Status>,
    pub refcount: Cell<u64>,
    pub fun: *mut dyn FnMut(*mut ()),
    pub out: *mut dyn Any,
    // fun_alloc: impl FnMut(&mut Option<T>),
    // out_alloc: Result<T, Box<dyn Any + Send + 'static>,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Status {
    Waiting,
    Queued,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Lifecycle {
    New,
    Running,
    Finished,
    Taken,
    OsThread,
}

impl Context {
    pub fn new<T, F>(fun: F, mut builder: Builder) -> io::Result<RcContext>
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
                stack: Stack::new(builder.stack_size)?,
                name: builder.name.take(),
                runtime: builder.runtime(),
                refcount: 1.into(),
                status: Cell::new(Status::Waiting),
                fun: fun_alloc as *mut dyn FnMut(*mut ()),
                lifecycle: Lifecycle::New.into(),
                layout,
                out,
            };
            ptr.cast::<Context>().write(cx);
            let cx = RcContext(NonNull::new(ptr.cast()).unwrap());
            Ok(cx.setup_registers())
        }
    }

    pub fn for_os_thread() -> RcContext {
        let mut cx = Self::new::<(), _>(|_| (), Builder::for_os_thread()).unwrap();
        cx.lifecycle.set(Lifecycle::OsThread);
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
