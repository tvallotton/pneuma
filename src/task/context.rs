use crate::debug_registers;

use super::{registers::Registers, stack::Stack};
use std::alloc::alloc;
use std::alloc::Layout;
use std::any::Any;
use std::arch::asm;
use std::mem::zeroed;
use std::ptr::NonNull;
/// The thread context as it was left before the switch.
///
/// # Allocation
/// It is import to remember that the context allocation
/// is extended to contain the closure and its output.

#[repr(C)]
pub struct Context {
    pub registers: Registers,
    pub stack: Stack,
    pub layout: Layout,
    pub status: Status,
    pub refcount: u64,
    pub fun: *mut dyn FnMut(*mut ()),
    pub out: *mut dyn Any,
    // fun_alloc: impl FnMut(&mut Option<T>),
    // out_alloc: Result<T, Box<dyn Any + Send + 'static>,
}
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub enum Status {
    New = 4,
    Running = 1,
    Finished = 2,
    Taken = 3,
}

impl Context {
    pub fn new<T, F>(size: usize, fun: F) -> NonNull<Context>
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
                stack: Stack::new(size),
                refcount: 1,
                fun: fun_alloc as *mut dyn FnMut(*mut ()),
                layout,
                status: Status::New,
                out,
            };
            ptr.cast::<Context>().write(cx);
            NonNull::new(ptr.cast()).unwrap()
        }
    }

    pub fn for_os_thread() -> NonNull<Context> {
        let mut cx = Self::new::<(), _>(0, |_| ());
        unsafe { cx.as_mut().status = Status::Running };
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
