use super::{registers::Registers, stack::Stack};
use std::alloc::alloc;
use std::alloc::Layout;
use std::any::Any;
use std::cell::Cell;
use std::mem::zeroed;
use std::ptr::NonNull;

/// The Context struct allocates all its fields along with the
/// type errased closure and the output parameter.

#[repr(C)]
pub struct Context {
    pub registers: Registers,
    pub stack: Stack,
    pub layout: Layout,
    pub status: Status,
    pub refcount: Cell<u64>,
    pub fun: *mut dyn FnMut(*mut ()),
    pub out: *mut dyn Any,
    // fun_alloc: impl FnMut(&mut Option<T>),
    // out_alloc: Result<T, Box<dyn Any + Send + Sync + 'static>,
}
#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
pub enum Status {
    New = 0,
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

            let fun_alloc = ptr.add(fun_offset).cast();
            *fun_alloc = fun;
            let out = ptr
                .add(out_offset)
                .cast::<Result<T, Box<dyn Any + Send + Sync + 'static>>>()
                as *mut dyn Any;

            *ptr.cast() = Context {
                registers: zeroed(),
                stack: Stack::new(size),
                refcount: Cell::new(1),
                fun: fun_alloc as *mut dyn FnMut(*mut ()),
                layout,
                status: Status::New,
                out,
            };

            NonNull::new(ptr.cast()).unwrap()
        }
    }

    // pub const fn take_output<T>(&mut self) -> Option<T> {
    //     let out = unsafe { &mut *self.out };

    // }

    pub extern "C" fn call_function(&mut self) {
        assert_eq!(self.status, Status::New);
        let f = unsafe { self.fun.as_mut().unwrap_unchecked() };
        self.status = Status::Running;
        f(self.out.cast());
    }
}

pub(crate) fn layout<T, F>() -> (Layout, usize, usize) {
    let raw_task = Layout::new::<Context>();
    let fun = Layout::new::<F>();
    let out = Layout::new::<Result<T, Box<dyn Any + Send + Sync + 'static>>>();
    let (layout, fun) = raw_task.extend(fun).unwrap();
    let (layout, out) = layout.extend(out).unwrap();
    (layout, fun, out)
}
