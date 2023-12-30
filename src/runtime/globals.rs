use super::Runtime;

thread_local! {
    static RUNTIME: Runtime =  Runtime::new();
}

pub fn current() -> Runtime {
    RUNTIME.with(|rt| rt.clone())
}
