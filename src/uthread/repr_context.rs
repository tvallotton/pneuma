use std::{
    alloc::Layout,
    any::Any,
    cell::UnsafeCell,
    io,
    mem::zeroed,
    ptr::NonNull,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicU8, Ordering},
        Mutex,
    },
};

use std::alloc::alloc;

use super::{
    builder::Builder, context::Context, lifecycle::OS_THREAD, registers::Registers,
    thread_id::UThreadId, UThread,
};
use pneuma::{sys::stack::Stack, uthread::lifecycle::NEW};

const LOCKED: bool = false;
const UNLOCKED: bool = !LOCKED;

#[repr(C)]
pub struct ReprContext {
    pub registers: UnsafeCell<Registers>,

    pub stack: Stack,

    pub lifecycle: AtomicU8,

    pub is_queued: AtomicBool,

    pub is_running: AtomicBool,

    pub panic_flag: AtomicU8,

    #[cfg(target_os = "linux")]
    pub io_uring_result: AtomicI64,

    pub refcount: AtomicU64,

    pub join_waker: Mutex<Option<UThread>>,

    pub fun: *mut dyn FnMut(*mut ()),

    pub out: *mut (),
    /// immutable
    pub name: Option<String>,
    /// immutable
    pub layout: Layout,
    /// immutable
    pub id: UThreadId,
}

impl ReprContext {
    /// Safety
    /// The context cannot outlive F and T's lifetime.
    pub unsafe fn new<'scope, T, F>(fun: F, mut builder: Builder) -> io::Result<Context>
    where
        F: FnMut(*mut ()) + 'scope,
        T: 'scope,
    {
        let (layout, cx, fun, out) = Self::setup_alloc::<'scope, F, T>(fun);

        cx.as_ptr().write(ReprContext {
            registers: zeroed(),
            stack: builder.stack()?,
            lifecycle: NEW.into(),
            is_queued: false.into(),
            is_running: false.into(),
            panic_flag: 0.into(),
            refcount: 1.into(),
            join_waker: Mutex::default(),
            name: builder.name.take(),
            id: UThreadId::new(),
            layout,
            fun,
            out,
            #[cfg(target_os = "linux")]
            io_uring_result: 0.into(),
        });
        Ok(Context { ptr: cx })
    }

    unsafe fn setup_alloc<'scope, F, T>(
        fun: F,
    ) -> (Layout, NonNull<Self>, *mut dyn FnMut(*mut ()), *mut ())
    where
        F: FnMut(*mut ()) + 'scope,
        T: 'scope,
    {
        let (layout, fun_offset, out_offset) = Self::layout::<T, F>();
        let ptr = alloc(layout);
        let ptr = NonNull::new(ptr).unwrap();
        let fun_alloc = ptr.as_ptr().add(fun_offset) as *mut F;
        fun_alloc.write(fun);

        let fun_alloc: *mut (dyn FnMut(*mut ()) + 'scope) = fun_alloc as _;
        let fun_alloc: *mut dyn FnMut(*mut ()) = fun_alloc as _;

        let out_alloc = ptr.as_ptr().add(out_offset).cast();

        (layout, ptr.cast(), fun_alloc, out_alloc)
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
        let cx = unsafe { Self::new::<(), _>(|_| (), Builder::for_os_thread()).unwrap() };
        cx.lifecycle.store(OS_THREAD, Ordering::Release);
        cx
    }
}
