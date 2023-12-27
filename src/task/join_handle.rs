use super::Task;

pub struct JoinHandle<T>(pub(crate) Task<T>);
