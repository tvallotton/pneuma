use std::{
    alloc::Layout,
    any::Any,
    cell::UnsafeCell,
    io,
    mem::zeroed,
    ptr::NonNull,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Mutex,
    },
};

use std::alloc::alloc;

use super::{
    builder::Builder, context::Context, lifecycle::OS_THREAD, registers::Registers,
    thread_id::UThreadId, UThread,
};
use crate::{sys::stack::Stack, uthread::lifecycle::NEW};

#[repr(C)]
pub struct ReprContext {
    pub registers: UnsafeCell<Registers>,

    pub stack: Stack,

    pub lifecycle: AtomicU8,

    pub is_queued: AtomicBool,

    pub is_running: AtomicBool,

    pub refcount: AtomicU64,

    pub join_waker: Mutex<Option<UThread>>,

    pub fun: *mut dyn FnMut(*mut ()),

    pub out: *mut dyn Any,
    /// immutable
    pub name: Option<String>,
    /// immutable
    pub layout: Layout,
    /// immutable
    pub id: UThreadId,
}

impl ReprContext {
    pub fn new<T, F>(fun: F, mut builder: Builder) -> io::Result<Context>
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        unsafe { Self::_new::<T, F>(fun, builder) }
    }

    pub unsafe fn _new<T, F>(fun: F, mut builder: Builder) -> io::Result<Context>
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        let (layout, cx, fun, out) = Self::setup_alloc::<_, T>(fun);

        cx.as_ptr().write(ReprContext {
            registers: zeroed(),
            stack: builder.stack()?,
            lifecycle: NEW.into(),
            is_queued: false.into(),
            is_running: false.into(),
            refcount: 1.into(),
            join_waker: Mutex::default(),
            name: builder.name.take(),
            id: UThreadId::new(),
            layout,
            fun,
            out,
        });
        Ok(Context { ptr: cx })
    }

    unsafe fn setup_alloc<F, T>(
        fun: F,
    ) -> (
        Layout,
        NonNull<ReprContext>,
        *mut dyn FnMut(*mut ()),
        *mut dyn Any,
    )
    where
        F: FnMut(*mut ()) + 'static,
        T: 'static,
    {
        let (layout, fun_offset, out_offset) = Self::layout::<T, F>();
        let ptr = alloc(layout);
        let ptr = NonNull::new(ptr).unwrap();
        let fun_alloc = ptr.as_ptr().add(fun_offset) as *mut F;
        fun_alloc.write(fun);
        let out_alloc =
            ptr.as_ptr()
                .add(out_offset)
                .cast::<Result<T, Box<dyn Any + Send + 'static>>>() as *mut dyn Any;

        return (layout, ptr.cast(), fun_alloc, out_alloc);
    }

    fn layout<T, F>() -> (Layout, usize, usize) {
        let raw_task = Layout::new::<ReprContext>();
        let fun = Layout::new::<F>();
        let out = Layout::new::<Result<T, Box<dyn Any + Send + 'static>>>();
        let (layout, fun) = raw_task.extend(fun).unwrap();
        let (layout, out) = layout.extend(out).unwrap();
        (layout, fun, out)
    }

    pub fn for_os_thread() -> Context {
        let cx = Self::new::<(), _>(|_| (), Builder::for_os_thread()).unwrap();
        cx.lifecycle.store(OS_THREAD, Ordering::Release);
        cx
    }
}

pub(crate) fn layout<T, F>() -> (Layout, usize, usize) {
    let raw_task = Layout::new::<ReprContext>();
    let fun = Layout::new::<F>();
    let out = Layout::new::<Result<T, Box<dyn Any + Send + 'static>>>();
    let (layout, fun) = raw_task.extend(fun).unwrap();
    let (layout, out) = layout.extend(out).unwrap();
    (layout, fun, out)
}
