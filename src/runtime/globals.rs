use super::Runtime;
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;

thread_local! {
    static RUNTIME: ManuallyDrop<UnsafeCell<Runtime>> =  {
        ON_DROP.with(|_| ());
        let _ = std::backtrace::Backtrace::force_capture();
        let runtime = Runtime::new();
        let runtime = UnsafeCell::new(runtime);
        ManuallyDrop::new(runtime)
    };
    static ON_DROP: OnDrop = const { OnDrop };
}

pub fn current() -> Runtime {
    RUNTIME.with(|rt| {
        let rt = unsafe { &*rt.get() };

        rt.clone()
    })
}

struct OnDrop;
impl Drop for OnDrop {
    fn drop(&mut self) {
        RUNTIME.with(|rt| unsafe {
            rt.get().read().shutdown();
        });
    }
}
