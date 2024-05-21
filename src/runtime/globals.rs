use super::Runtime;
use std::sync::OnceLock;

pub(crate) static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn current() -> &'static Runtime {
    dbg!();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}
